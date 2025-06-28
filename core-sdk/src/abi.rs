use codec::{Decode, Encode};
use scale_info::{form::PortableForm, MetaType, Registry, Type};

#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Encode, Decode, Eq, PartialEq)]
pub struct ApiEntry {
    pub name: String,
    pub method: String,
    pub param_types: Vec<u32>,
    pub return_type: u32,
}

pub struct ApiRegistry {
    pub types: Registry,
    pub entries: Vec<ApiEntry>,
}

#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize, Decode))]
#[derive(Debug, Clone, Encode, Eq, PartialEq)]
pub struct JsonAbi {
    pub functions: Vec<ApiEntry>,
    pub types: Vec<AbiType>,
}

impl JsonAbi {
    #[cfg(feature = "std")]
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).expect("Failed to serialize JsonAbi")
    }
}

#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize, Decode))]
#[derive(Debug, Clone, Encode, Eq, PartialEq)]
pub struct AbiType {
    pub id: u32,
    pub ty: Type<PortableForm>,
}

impl ApiRegistry {
    pub fn new() -> Self {
        Self {
            types: Registry::new(),
            entries: Vec::new(),
        }
    }

    pub fn register_api(
        &mut self,
        name: String,
        method: String,
        param_types: Vec<MetaType>,
        return_type: MetaType,
    ) {
        let param_types = self
            .types
            .register_types(param_types)
            .into_iter()
            .map(|ty| ty.id)
            .collect();
        let return_type = self.types.register_type(&return_type).id;
        let entry = ApiEntry {
            name,
            method,
            param_types,
            return_type,
        };
        self.entries.push(entry);
    }

    pub fn dump_abi(&self) -> JsonAbi {
        let types = self
            .types
            .types()
            .map(|(symbol, ty)| AbiType {
                id: symbol.id,
                ty: ty.clone(),
            })
            .collect::<Vec<_>>();
        JsonAbi {
            functions: self.entries.clone(),
            types,
        }
    }
}
