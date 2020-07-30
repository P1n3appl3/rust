//! Rustdoc's JSON output interface
//!
//! These types are the public API exposed through the `--output-format json` flag. The [`Crate`][]
//! struct is the root of the JSON blob and all other items are contained within.

use std::path::PathBuf;

use rustc_data_structures::fx::FxHashMap;
use serde::Serialize;

/// A `Crate` is the root of the emitted JSON blob. It contains all type/documentation information
/// about the language items in the local crate, as well as info about external items to allow
/// tools to find or link to them.
#[derive(Clone, Debug, Serialize)]
pub struct Crate {
    /// The id of the root [`Module`][] item of the local crate.
    pub root: Id,
    /// The version string given to `--crate-version`, if any.
    pub version: Option<String>,
    /// Whether or not the output includes private items.
    pub includes_private: bool,
    /// A collection of all items in the local crate as well as some external traits and their
    /// items that are referenced locally.
    pub index: FxHashMap<Id, Item>,
    /// Maps ids to fully qualified paths (e.g. `["std", "io", "lazy", "Lazy"]` for
    /// `std::io::lazy::Lazy`) as well as their `ItemKind`
    pub paths: FxHashMap<Id, ItemSummary>,
    /// Maps `crate_num` of items to a crate name and html_root_url if it exists
    pub external_crates: FxHashMap<u32, ExternalCrate>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ExternalCrate {
    pub name: String,
    pub html_root_url: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ItemSummary {
    pub crate_num: u32,
    pub path: Vec<String>,
    pub kind: ItemKind,
}

#[derive(Clone, Debug, Serialize)]
pub struct Item {
    /// This can be used as a key to the `external_crates` map of [`Crate`][] to see which crate
    /// this item came from.
    pub crate_num: u32,
    /// Some items such as impls don't have names.
    pub name: Option<String>,
    /// The source location of this item. May not be present if it came from a macro expansion,
    /// inline assembly, other "virtual" files
    pub source: Option<Span>,
    pub visibility: Visibility,
    pub docs: String,
    pub links: Vec<(String, Option<Id>, Option<String>)>,
    pub attrs: Vec<String>,
    pub deprecation: Option<Deprecation>,
    pub kind: ItemKind,
    pub inner: ItemEnum,
    // TODO: should we stringify the cfg attrs as well, or should we preserve their structure so
    // the consumer doesn't have to parse an arbitrarily nested tree to figure out what platforms
    // the item is available on?
    // TODO: should we have a "stability" field if it's only used by the standard library?
}

#[derive(Clone, Debug, Serialize)]
pub struct Span {
    /// The path to the source file for this span relative to the crate root.
    pub filename: PathBuf,
    /// Zero indexed Line and Column of the first character of the `Span`
    pub begin: (usize, usize),
    /// Zero indexed Line and Column of the last character of the `Span`
    pub end: (usize, usize),
}

#[derive(Clone, Debug, Serialize)]
pub struct Deprecation {
    pub since: Option<String>,
    pub note: Option<String>,
}

#[serde(rename_all = "snake_case")]
#[serde(tag = "visibility", content = "restricted_path")]
#[derive(Clone, Debug, Serialize)]
pub enum Visibility {
    Public,
    Default,
    Crate,
    Restricted(Id, String),
}

#[serde(rename_all = "snake_case")]
#[derive(Clone, Debug, Serialize)]
pub enum GenericArgs {
    /// <'a, 32, B: Copy, C = u32>
    AngleBracketed { args: Vec<GenericArg>, bindings: Vec<TypeBinding> },
    /// Fn(A, B) -> C
    Parenthesized { inputs: Vec<Type>, output: Option<Type> },
}

#[serde(rename_all = "snake_case")]
#[derive(Clone, Debug, Serialize)]
pub enum GenericArg {
    Lifetime(String),
    Type(Type),
    Const(Constant),
}

#[derive(Clone, Debug, Serialize)]
pub struct Constant {
    #[serde(rename = "type")]
    pub type_: Type,
    pub expr: String,
    pub value: Option<String>,
    pub is_literal: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct TypeBinding {
    pub name: String,
    pub binding: TypeBindingKind,
}

#[serde(rename_all = "snake_case")]
#[derive(Clone, Debug, Serialize)]
pub enum TypeBindingKind {
    Equality(Type),
    Constraint(Vec<GenericBound>),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct Id(pub String);

#[serde(rename_all = "snake_case")]
#[derive(Clone, Debug, Serialize)]
pub enum ItemKind {
    Module,
    ExternCrate,
    Import,
    Struct,
    StructField,
    Union,
    Enum,
    Variant,
    Function,
    Typedef,
    OpaqueTy,
    Constant,
    Trait,
    TraitAlias,
    Method,
    Impl,
    Static,
    ForeignType,
    Macro,
    ProcAttribute,
    ProcDerive,
    AssocConst,
    AssocType,
    Primitive,
    Keyword,
}

#[serde(untagged)]
#[derive(Clone, Debug, Serialize)]
pub enum ItemEnum {
    ModuleItem(Module),
    ExternCrateItem {
        name: String,
        rename: Option<String>,
    },
    ImportItem(Import),

    StructItem(Struct),
    StructFieldItem(Type),
    EnumItem(Enum),
    VariantItem(Variant),

    FunctionItem(Function),

    TypedefItem(Typedef),
    OpaqueTyItem(OpaqueTy),
    ConstantItem(Constant),

    TraitItem(Trait),
    TraitAliasItem(TraitAlias),
    MethodItem(Method),
    ImplItem(Impl),

    StaticItem(Static),

    /// `type`s from an extern block
    ForeignTypeItem,

    /// Declarative macro_rules! macro
    MacroItem(String),
    ProcMacroItem(ProcMacro),

    AssocConstItem {
        #[serde(rename = "type")]
        type_: Type,
        default: Option<String>,
    },
    AssocTypeItem {
        bounds: Vec<GenericBound>,
        default: Option<Type>,
    },

    /// An item that has been stripped by a rustdoc pass
    StrippedItem(Box<ItemEnum>),
}

#[derive(Clone, Debug, Serialize)]
pub struct Module {
    pub is_crate: bool,
    pub items: Vec<Id>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Struct {
    pub struct_type: StructType,
    pub generics: Generics,
    pub fields_stripped: bool,
    pub fields: Vec<Id>,
    pub impls: Vec<Id>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Enum {
    pub generics: Generics,
    pub variants_stripped: bool,
    pub variants: Vec<Id>,
    pub impls: Vec<Id>,
}

#[serde(rename_all = "snake_case")]
#[serde(tag = "variant_kind", content = "inner")]
#[derive(Clone, Debug, Serialize)]
pub enum Variant {
    Plain,
    Tuple(Vec<Type>),
    Struct(Struct),
}

#[serde(rename_all = "snake_case")]
#[derive(Clone, Debug, Serialize)]
pub enum StructType {
    Plain,
    Tuple,
    Unit,
}

#[derive(Clone, Debug, Serialize)]
pub struct Function {
    pub decl: FnDecl,
    pub generics: Generics,
    pub header: String,
    pub abi: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct Method {
    pub decl: FnDecl,
    pub generics: Generics,
    pub header: String,
    pub has_body: bool,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct Generics {
    pub params: Vec<GenericParamDef>,
    pub where_predicates: Vec<WherePredicate>,
}

#[derive(Clone, Debug, Serialize)]
pub struct GenericParamDef {
    pub name: String,
    pub kind: GenericParamDefKind,
}

#[serde(rename_all = "snake_case")]
#[derive(Clone, Debug, Serialize)]
pub enum GenericParamDefKind {
    Lifetime,
    Type { bounds: Vec<GenericBound>, default: Option<Type> },
    Const(Type),
}

#[serde(rename_all = "snake_case")]
#[derive(Clone, Debug, Serialize)]
pub enum WherePredicate {
    BoundPredicate { ty: Type, bounds: Vec<GenericBound> },
    RegionPredicate { lifetime: String, bounds: Vec<GenericBound> },
    EqPredicate { lhs: Type, rhs: Type },
}

#[serde(rename_all = "snake_case")]
#[derive(Clone, Debug, Serialize)]
pub enum GenericBound {
    TraitBound {
        #[serde(rename = "trait")]
        trait_: Type,
        /// Used for HRTBs
        generic_params: Vec<GenericParamDef>,
        modifier: TraitBoundModifier,
    },
    Outlives(String),
}

#[serde(rename_all = "snake_case")]
#[derive(Clone, Debug, Serialize)]
pub enum TraitBoundModifier {
    None,
    Maybe,
    MaybeConst,
}

#[serde(rename_all = "snake_case")]
#[serde(tag = "kind", content = "inner")]
#[derive(Clone, Debug, Serialize)]
pub enum Type {
    /// Structs, enums, and traits
    ResolvedPath {
        name: String,
        id: Id,
        args: Box<Option<GenericArgs>>,
        param_names: Vec<GenericBound>,
    },
    /// Parameterized types
    Generic(String),
    /// Fixed-size numeric types (plus int/usize/float), char, arrays, slices, and tuples
    Primitive(String),
    /// `extern "ABI" fn`
    FunctionPointer(Box<FunctionPointer>),
    /// `(String, u32, Box<usize>)`
    Tuple(Vec<Type>),
    /// `[u32]`
    Slice(Box<Type>),
    /// [u32; 15]
    Array {
        #[serde(rename = "type")]
        type_: Box<Type>,
        len: String,
    },
    /// `impl TraitA + TraitB + ...`
    ImplTrait(Vec<GenericBound>),
    /// `!`
    Never,
    /// `_`
    Infer,
    /// `*mut u32`, `*u8`, etc.
    RawPointer {
        mutable: bool,
        #[serde(rename = "type")]
        type_: Box<Type>,
    },
    /// `&'a mut String`, `&str`, etc.
    BorrowedRef {
        lifetime: Option<String>,
        mutable: bool,
        #[serde(rename = "type")]
        type_: Box<Type>,
    },
    /// `<Type as Trait>::Name` or associated types like `T::Item` where `T: Iterator`
    QualifiedPath {
        name: String,
        self_type: Box<Type>,
        #[serde(rename = "trait")]
        trait_: Box<Type>,
    },
}

#[derive(Clone, Debug, Serialize)]
pub struct FunctionPointer {
    pub is_unsafe: bool,
    pub generic_params: Vec<GenericParamDef>,
    pub decl: FnDecl,
    pub abi: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct FnDecl {
    pub inputs: Vec<(String, Type)>,
    pub output: Option<Type>,
    pub c_variadic: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct Trait {
    pub is_auto: bool,
    pub is_unsafe: bool,
    pub items: Vec<Id>,
    pub generics: Generics,
    pub bounds: Vec<GenericBound>,
    pub implementors: Vec<Id>,
}

#[derive(Clone, Debug, Serialize)]
pub struct TraitAlias {
    pub generics: Generics,
    pub bounds: Vec<GenericBound>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Impl {
    pub is_unsafe: bool,
    pub generics: Generics,
    pub provided_trait_methods: Vec<String>,
    #[serde(rename = "trait")]
    pub trait_: Option<Type>,
    #[serde(rename = "for")]
    pub for_: Type,
    pub items: Vec<Id>,
    pub negative: bool,
    pub synthetic: bool,
    pub blanket_impl: Option<Type>,
}

// TODO: this is currently broken because imports have the same ID as the module that contains
// them. The only obvious fix is to modify the clean types to renumber imports so that IDs are
// actually unique.
#[serde(rename_all = "snake_case")]
#[derive(Clone, Debug, Serialize)]
pub struct Import {
    /// The full path being imported.
    pub source: String,
    /// May be different from the last segment of `source` when renaming imports:
    /// `use source as name;`
    pub name: String,
    /// The ID of the item being imported.
    pub id: Option<Id>, // TODO: when is this None?
    /// Whether this import uses a glob: `use source::*;`
    pub glob: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct ProcMacro {
    pub kind: MacroKind,
    pub helpers: Vec<String>,
}

#[serde(rename_all = "snake_case")]
#[derive(Clone, Debug, Serialize)]
pub enum MacroKind {
    /// A bang macro `foo!()`.
    Bang,
    /// An attribute macro `#[foo]`.
    Attr,
    /// A derive macro `#[derive(Foo)]`
    Derive,
}

#[derive(Clone, Debug, Serialize)]
pub struct Typedef {
    #[serde(rename = "type")]
    pub type_: Type,
    pub generics: Generics,
}

#[derive(Clone, Debug, Serialize)]
pub struct OpaqueTy {
    pub bounds: Vec<GenericBound>,
    pub generics: Generics,
}

#[derive(Clone, Debug, Serialize)]
pub struct Static {
    #[serde(rename = "type")]
    pub type_: Type,
    pub mutable: bool,
    pub expr: String,
}
