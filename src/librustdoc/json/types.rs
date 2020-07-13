use std::num::NonZeroU32;
use std::path::PathBuf;

use rustc_data_structures::fx::FxHashMap;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct Crate {
    pub root: Id,
    pub version: Option<String>,
    pub includes_private: bool,
    pub index: FxHashMap<Id, Item>,
    pub type_to_trait_impls: FxHashMap<Id, Vec<Id>>,
    pub trait_to_implementors: FxHashMap<Id, Vec<Id>>,
    /// Maps `crate_num` of items to crate names and html_root_url if it exists
    pub extern_crates: FxHashMap<u32, (String, Option<String>)>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Item {
    pub id: Id,
    pub crate_num: u32,
    pub name: Option<String>,
    pub source: Span,
    pub visibility: Visibility,
    pub docs: Option<String>,
    pub kind: ItemKind,
    pub inner: ItemEnum,
    // TODO: the `Attributes` struct defers to compiler internal symbols. seems like it would be
    // hard to expose arbitrary ones so we should either special case things like `cfg` that matter
    // for docs or just stringify all non-doc attributes to let the user deal with them
    // pub attrs: Attributes,
    // TODO: should we support this if it's only used by `std`?
    // pub stability: Option<Stability>,
    // TODO: why is this necessary when stability contains one?
    // pub deprecation: Option<Deprecation>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Span {
    pub filename: Option<PathBuf>,
    /// (Line, Column) of the first character of the `Span`
    pub begin: (usize, usize),
    /// (Line, Column) of the last character of the `Span`
    pub end: (usize, usize),
}

#[serde(rename_all = "snake_case")]
#[derive(Clone, Debug, Serialize)]
pub enum Visibility {
    Public,
    Inherited,
    Crate,
    Restricted(Id, Path),
}

#[derive(Clone, Debug, Serialize)]
pub struct Path {
    pub global: bool,
    pub segments: Vec<PathSegment>,
}

#[derive(Clone, Debug, Serialize)]
pub struct PathSegment {
    pub name: String,
    pub args: GenericArgs,
}

#[serde(rename_all = "snake_case")]
#[derive(Clone, Debug, Serialize)]
pub enum GenericArgs {
    AngleBracketed { args: Vec<GenericArg>, bindings: Vec<TypeBinding> },
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
    pub kind: TypeBindingKind,
}

#[serde(rename_all = "snake_case")]
#[derive(Clone, Debug, Serialize)]
pub enum TypeBindingKind {
    Equality(Type),
    Constraint(Vec<GenericBound>),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct Id(pub String);

#[derive(Clone, Debug, Serialize)]
pub struct Stability {
    pub stable: bool,
    pub feature: Option<String>,
    pub since: String,
    pub deprecation: Option<Deprecation>,
    pub unstable_reason: Option<String>,
    pub issue: Option<NonZeroU32>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Deprecation {
    pub since: Option<String>,
    pub note: Option<String>,
}

#[serde(rename_all = "snake_case")]
#[derive(Clone, Debug, Serialize)]
pub enum ItemKind {
    Module,
    ExternCrate,
    Import,
    Struct,
    StructField,
    Enum,
    Variant,
    Function,
    ForeignFunction,
    Typedef,
    OpaqueTy,
    Constant,
    Trait,
    TraitAlias,
    Method,
    Impl,
    Static,
    ForeignStatic,
    ForeignType,
    Macro,
    ProcMacro,
    AssocConst,
    AssocType,
    Stripped,
}

#[serde(rename_all = "snake_case")]
#[derive(Clone, Debug, Serialize)]
pub enum ItemEnum {
    ModuleItem(Module),
    ExternCrateItem(String, Option<String>),
    ImportItem(Import),

    StructItem(Struct),
    StructFieldItem(Type),
    EnumItem(Enum),
    VariantItem(Variant),

    FunctionItem(Function),
    /// `fn`s from an extern block
    ForeignFunctionItem(Function),

    TypedefItem(Typedef, bool /* is associated type */),
    OpaqueTyItem(OpaqueTy, bool /* is associated type */),
    ConstantItem(Constant),

    TraitItem(Trait),
    TraitAliasItem(TraitAlias),
    MethodItem(Method),
    ImplItem(Impl),

    StaticItem(Static),
    /// `static`s from an extern block
    ForeignStaticItem(Static),
    /// `type`s from an extern block
    ForeignTypeItem,

    MacroItem(Macro),
    ProcMacroItem(ProcMacro),

    AssocConstItem(Type, Option<String>),
    AssocTypeItem(Vec<GenericBound>, Option<Type>),

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
    pub fields: Vec<Item>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Enum {
    pub generics: Generics,
    pub variants_stripped: bool,
    pub variants: Vec<Item>,
}

#[serde(rename_all = "snake_case")]
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
    pub header: FnHeader,
}

#[derive(Clone, Debug, Serialize)]
pub struct Method {
    pub header: FnHeader,
    pub decl: FnDecl,
    pub generics: Generics,
    pub has_body: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct FnHeader {
    pub is_unsafe: bool,
    pub is_const: bool,
    pub is_async: bool,
    pub abi: String,
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
    Type {
        id: Id,
        bounds: Vec<GenericBound>,
        default: Option<Type>,
        // synthetic: bool, // TODO: check if necessary
    },
    Const {
        id: Id,
        ty: Type,
    },
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
    TraitBound(PolyTrait, TraitBoundModifier),
    Outlives(String),
}

#[derive(Clone, Debug, Serialize)]
pub struct PolyTrait {
    #[serde(rename = "trait")]
    pub trait_: Type,
    pub generic_params: Vec<GenericParamDef>,
}

#[serde(rename_all = "snake_case")]
#[derive(Clone, Debug, Serialize)]
pub enum TraitBoundModifier {
    None,
    Maybe,
    MaybeConst,
}

#[serde(rename_all = "snake_case")]
#[derive(Clone, Debug, Serialize)]
pub enum PrimitiveType {
    Isize,
    I8,
    I16,
    I32,
    I64,
    I128,
    Usize,
    U8,
    U16,
    U32,
    U64,
    U128,
    F32,
    F64,
    Char,
    Bool,
    Str,
    Slice,
    Array,
    Tuple,
    Unit,
    RawPointer,
    Reference,
    Fn,
    Never,
}

#[serde(rename_all = "snake_case")]
#[derive(Clone, Debug, Serialize)]
pub enum Type {
    /// Structs/enums/traits (most that would be an `hir::TyKind::Path`).
    ResolvedPath {
        path: Path,
        param_names: Option<Vec<GenericBound>>,
        id: Id,
        /// `true` if is a `T::Name` path for associated types.
        is_generic: bool,
    },
    /// For parameterized types, so the consumer of the JSON don't go
    /// looking for types which don't exist anywhere.
    Generic(String),
    /// Primitives are the fixed-size numeric types (plus int/usize/float), char,
    /// arrays, slices, and tuples.
    Primitive(PrimitiveType),
    /// `extern "ABI" fn`
    BareFunction(Box<BareFunctionDecl>),
    Tuple(Vec<Type>),
    Slice(Box<Type>),
    // Second field is stringified length
    Array(Box<Type>, String),
    ImplTrait(Vec<GenericBound>), // `impl TraitA + TraitB + ...`
    Never,
    Infer, // `_`
    RawPointer {
        mutable: bool,
        #[serde(rename = "type")]
        type_: Box<Type>,
    },
    BorrowedRef {
        lifetime: Option<String>,
        mutable: bool,
        #[serde(rename = "type")]
        type_: Box<Type>,
    },
    // `<Type as Trait>::Name`
    QPath {
        name: String,
        self_type: Box<Type>,
        #[serde(rename = "trait")]
        trait_: Box<Type>,
    },
}

#[derive(Clone, Debug, Serialize)]
pub struct BareFunctionDecl {
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
    pub items: Vec<Item>,
    pub generics: Generics,
    pub bounds: Vec<GenericBound>,
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
    pub items: Vec<Item>,
    pub negative: bool,
    pub synthetic: bool,
    pub blanket_impl: Option<Type>,
}

#[serde(rename_all = "snake_case")]
#[derive(Clone, Debug, Serialize)]
pub enum Import {
    // use source as str;
    Simple(String, ImportSource),
    // use source::*;
    Glob(ImportSource),
}

#[derive(Clone, Debug, Serialize)]
pub struct ImportSource {
    pub path: Path,
    pub id: Option<Id>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Macro {
    pub source: String,
    pub imported_from: Option<String>,
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
    // Type of target item.
    pub item_type: Option<Type>,
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
