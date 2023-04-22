use core::panic;

use crate::{
    grammar::{GHeaderFileItem, GMarker, GType, GTypeCategory},
    CHeaderDirectory,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Error {
    BadImport,
    BadObject,
    BadProperty,
    BadType,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct TypeInfo {
    pub variant: TypeVariant,
    pub is_constant: bool,
    pub is_nullable: bool,
    pub is_pointer: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum TypeVariant {
    Void,
    Bool,
    Int8,
    Int16,
    Int32,
    Int64,
    Uint8,
    Uint16,
    Uint32,
    Uint64,
    Float32,
    Float64,
    Struct(String),
    Enum(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub imports: Vec<ImportInfo>,
    pub structs: Vec<StructInfo>,
    pub enums: Vec<EnumInfo>,
    pub functions: Vec<MethodInfo>,
    pub properties: Vec<PropertyInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportInfo {
    // Expressed as directories plus the final file.
    // E.g. `to/some/file.h` ~= ["to", "some", "file.h"]
    pub path: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumInfo {
    pub name: String,
    pub is_public: bool,
    pub variants: Vec<(String, Option<usize>)>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructInfo {
    pub name: String,
    pub is_public: bool,
    pub fields: Vec<(String, TypeInfo)>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyInfo {
    pub name: String,
    pub is_public: bool,
    pub is_static: bool,
    pub return_type: TypeInfo,
    pub comments: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodInfo {
    pub name: String,
    pub is_public: bool,
    pub is_static: bool,
    pub params: Vec<ParamInfo>,
    pub return_type: TypeInfo,
    pub comments: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamInfo {
    pub name: String,
    pub ty: TypeInfo,
}

// NOTE: This function is temporary
pub fn process_c_header_dir(dir: &CHeaderDirectory) {
    for (path, items) in &dir.map {
        //println!("### {:?}", path);

        let file_name = path
            .to_str()
            .unwrap()
            .split("/")
            .last()
            .unwrap()
            .strip_suffix(".h")
            .unwrap();

        let mut file_info = FileInfo {
            name: file_name.to_string(),
            imports: vec![],
            structs: vec![],
            enums: vec![],
            functions: vec![],
            properties: vec![],
        };

        for item in items {
            match item {
                GHeaderFileItem::StructIndicator(decl) => {
                    file_info.structs.push(StructInfo {
                        name: decl.name.0.0.clone(),
                        is_public: true,
                        fields: vec![],
                        tags: vec![],
                    });
                }
                GHeaderFileItem::StructDecl(decl) => {
                    let x = StructInfo::from_g_type(decl).unwrap();
                    file_info.structs.push(x);
                }
                GHeaderFileItem::EnumDecl(decl) => {
                    let x = EnumInfo::from_g_type(decl).unwrap();
                    file_info.enums.push(x);
                }
                GHeaderFileItem::FunctionDecl(decl) => {
                    if decl.name.0.contains("CreateWith") || decl.name.0.contains("Delete") {
                        continue;
                    }

                    if decl.markers.0.contains(&GMarker::TwExportMethod)
                        || decl.markers.0.contains(&GMarker::TwExportStaticMethod)
                    {
                        let x = MethodInfo::from_g_type(&Some(file_name.to_string()), decl).unwrap();
                        file_info.functions.push(x);
                    }
                }
                _ => {},
            }
        }

        let content = serde_json::to_string_pretty(&file_info).unwrap();
        let mut file = std::fs::File::create(format!("out/{}.json", file_name)).unwrap();
        std::io::Write::write(&mut file, content.as_bytes()).unwrap();
    }
}

pub fn extract_custom(ty: &GType) -> Option<String> {
    match ty {
        GType::Mutable(cat) | GType::Const(cat) | GType::Extern(cat) => {
            if let GTypeCategory::Unrecognized(keyword) = cat {
                Some(keyword.0.clone())
            } else {
                None
            }
        }
    }
}

#[test]
#[ignore]
fn test_manifest() {
    let path = std::path::Path::new("../include/");
    let dir = crate::parse(&path).unwrap();
    process_c_header_dir(&dir);
}
