use core::fmt;
use std::marker::PhantomData;

use serde::Deserialize;
use serde::de::Deserializer;
use serde::de::EnumAccess;
use serde::de::MapAccess;
use serde::de::Visitor;
use serde::de::value::EnumAccessDeserializer;
use serde::de::value::MapAccessDeserializer;

use crate::schema::Metadata;
use crate::schema::StringMap;

pub type DefinitionTree<T> = StringMap<DefinitionNode<T>>;
pub type MetadataTree = StringMap<MetadataNode>;

// --- single generic collapse, replaces the old `collapse`/`collapse_into` pair ---
pub fn collapse<N: TreeNode>(tree: StringMap<N>) -> StringMap<N::Leaf> {
    let mut result = StringMap::new();
    collapse_into(tree, String::new(), &mut result);
    result
}

fn collapse_into<N: TreeNode>(tree: StringMap<N>, prefix: String, result: &mut StringMap<N::Leaf>) {
    for (key, node) in tree {
        let full_key = if prefix.is_empty() {
            key
        } else {
            format!("{}.{}", prefix, key)
        };

        match node.into_leaf_or_branch() {
            Ok(value) => {
                result.insert(full_key, value);
            }
            Err(children) => {
                collapse_into(children, full_key, result);
            }
        }
    }
}

// --- generic flatten support ---
pub trait TreeNode {
    type Leaf;

    fn into_leaf_or_branch(self) -> Result<Self::Leaf, StringMap<Self>>
    where
        Self: Sized;
}

impl<V> TreeNode for DefinitionNode<V> {
    type Leaf = V;

    fn into_leaf_or_branch(self) -> Result<V, StringMap<Self>> {
        match self {
            DefinitionNode::Leaf(v) => Ok(v),
            DefinitionNode::Branch(children) => Err(children),
        }
    }
}

impl TreeNode for MetadataNode {
    type Leaf = Metadata;

    fn into_leaf_or_branch(self) -> Result<Metadata, StringMap<Self>> {
        match self {
            MetadataNode::Leaf(v) => Ok(v),
            MetadataNode::Branch(children) => Err(children),
        }
    }
}

// --- definition node ---
#[derive(Debug, Clone)]
pub enum DefinitionNode<V> {
    Branch(StringMap<DefinitionNode<V>>),
    Leaf(V),
}

impl<'de, V> Deserialize<'de> for DefinitionNode<V>
where
    V: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct KindVisitor<V>(PhantomData<V>);

        impl<'de, V> Visitor<'de> for KindVisitor<V>
        where
            V: Deserialize<'de>,
        {
            type Value = DefinitionNode<V>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a property group or tagged value")
            }

            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                let leaf = V::deserialize(EnumAccessDeserializer::new(data))?;
                Ok(DefinitionNode::Leaf(leaf))
            }

            fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let branch = StringMap::deserialize(MapAccessDeserializer::new(map))?;
                Ok(DefinitionNode::Branch(branch))
            }
        }

        deserializer.deserialize_any(KindVisitor(PhantomData))
    }
}

// --- metadata node ---
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum MetadataNode {
    Branch(StringMap<MetadataNode>),
    Leaf(Metadata),
}
