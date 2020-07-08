use std::num::NonZeroU32;
use std::path::PathBuf;

use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct Item {
    pub id: DefId,
    pub name: Option<String>,
    pub source: Span,
    pub visibility: Visibility,
    pub docs: Option<String>,
    // TODO: the `Attributes` struct defers to compiler internal symbols. seems like it would be
    // hard to expose arbitrary ones so we should either special case things like `cfg` that matter
    // for docs or just stringify all non-doc attributes to let the user deal with them as they
    // please
    // pub attrs: Attributes,
    pub inner: ItemEnum,
    // TODO: should we support this if it's only used by `std`
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

#[derive(Clone, PartialEq, Eq, Debug, Serialize)]
pub enum Visibility {
    Public,
    Inherited,
    Crate,
    Restricted(DefId, Path),
}

#[derive(Clone, PartialEq, Eq, Debug, Hash, Serialize)]
pub struct Path {
    pub global: bool,
    pub segments: Vec<PathSegment>,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash, Serialize)]
pub struct PathSegment {
    pub name: String,
    pub args: GenericArgs,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash, Serialize)]
pub enum GenericArgs {
    AngleBracketed { args: Vec<GenericArg>, bindings: Vec<TypeBinding> },
    Parenthesized { inputs: Vec<Type>, output: Option<Type> },
}

#[derive(Clone, PartialEq, Eq, Debug, Hash, Serialize)]
pub enum GenericArg {
    Lifetime(String),
    Type(Type),
    Const(Constant),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize)]
pub struct Constant {
    pub type_: Type,
    pub expr: String,
    pub value: Option<String>,
    pub is_literal: bool,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash, Serialize)]
pub struct TypeBinding {
    pub name: String,
    pub kind: TypeBindingKind,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash, Serialize)]
pub enum TypeBindingKind {
    Equality(Type),
    Constraint(Vec<GenericBound>),
}

#[derive(Clone, PartialEq, Eq, Debug, PartialOrd, Ord, Hash, Copy, Serialize)]
pub struct DefId(pub u32, pub u32);

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

#[derive(Clone, Debug, Serialize)]
pub enum ItemEnum {
    ModuleItem(Module),
    PrimitiveItem(PrimitiveType),
    StructItem(Struct),
    FunctionItem(Function),
    // EnumItem(Enum),
    // TypedefItem(Typedef, bool /* is associated type */),
    // ConstantItem(Constant),
    // TraitItem(Trait),
    // ImplItem(Impl),
    // /// A method signature only. Used for required methods in traits (ie,
    // /// non-default-methods).
    // TyMethodItem(TyMethod),
    // /// A method with a body.
    // MethodItem(Method),
    // StructFieldItem(Type),
    // VariantItem(Variant),
    // KeywordItem(String),

    // ExternCrateItem(String, Option<String>),
    // ImportItem(Import),
    // UnionItem(Union),
    // OpaqueTyItem(OpaqueTy, bool /* is associated type */),
    // StaticItem(Static),
    // TraitAliasItem(TraitAlias),
    // /// `fn`s from an extern block
    // ForeignFunctionItem(Function),
    // /// `static`s from an extern block
    // ForeignStaticItem(Static),
    // /// `type`s from an extern block
    // ForeignTypeItem,
    // MacroItem(Macro),
    // ProcMacroItem(ProcMacro),
    // AssocConstItem(Type, Option<String>),
    // AssocTypeItem(Vec<GenericBound>, Option<Type>),
    // /// An item that has been stripped by a rustdoc pass
    // StrippedItem(Box<ItemEnum>),
}

#[derive(Clone, Debug, Serialize)]
pub struct Module {
    pub is_crate: bool,
    pub items: Vec<Item>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Struct {
    pub struct_type: StructType,
    pub generics: Generics,
    pub fields_stripped: bool,
    pub fields: Vec<Item>,
}

#[derive(Debug, Clone, Copy, Serialize)]
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

#[derive(Clone, PartialEq, Eq, Debug, Hash, Serialize)]
pub struct GenericParamDef {
    pub name: String,
    pub kind: GenericParamDefKind,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash, Serialize)]
pub enum GenericParamDefKind {
    Lifetime,
    Type {
        did: DefId,
        bounds: Vec<GenericBound>,
        default: Option<Type>,
        // synthetic: bool, // TODO: check if necessary
    },
    Const {
        did: DefId,
        ty: Type,
    },
}

#[derive(Clone, Debug, Serialize)]
pub enum WherePredicate {
    BoundPredicate { ty: Type, bounds: Vec<GenericBound> },
    RegionPredicate { lifetime: String, bounds: Vec<GenericBound> },
    EqPredicate { lhs: Type, rhs: Type },
}

#[derive(Clone, PartialEq, Eq, Debug, Hash, Serialize)]
pub enum GenericBound {
    TraitBound(PolyTrait, TraitBoundModifier),
    Outlives(String),
}

#[derive(Clone, PartialEq, Eq, Debug, Hash, Serialize)]
pub struct PolyTrait {
    pub trait_: Type,
    pub generic_params: Vec<GenericParamDef>,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Serialize)]
pub enum TraitBoundModifier {
    None,
    Maybe,
    MaybeConst,
}

#[derive(Clone, PartialEq, Eq, Hash, Copy, Debug, Serialize)]
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

#[derive(Clone, PartialEq, Eq, Debug, Hash, Serialize)]
pub enum Type {
    /// Structs/enums/traits (most that would be an `hir::TyKind::Path`).
    ResolvedPath {
        path: Path,
        param_names: Option<Vec<GenericBound>>,
        did: DefId,
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
    Array(Box<Type>, String),
    ImplTrait(Vec<GenericBound>), // `impl TraitA + TraitB + ...`
    Never,
    Infer, // `_`
    RawPointer {
        mutable: bool,
        type_: Box<Type>,
    },
    BorrowedRef {
        lifetime: Option<String>,
        mutable: bool,
        type_: Box<Type>,
    },
    // `<Type as Trait>::Name`
    QPath {
        name: String,
        self_type: Box<Type>,
        trait_: Box<Type>,
    },
}

#[derive(Clone, PartialEq, Eq, Debug, Hash, Serialize)]
pub struct BareFunctionDecl {
    pub unsafe_: bool,
    pub generic_params: Vec<GenericParamDef>,
    pub decl: FnDecl,
    pub abi: String,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash, Serialize)]
pub struct FnDecl {
    pub inputs: Vec<(String, Type)>,
    pub output: Option<Type>,
    pub c_variadic: bool,
}

#[derive(Clone, PartialEq, Eq, Hash, Copy, Debug, Serialize)]
pub enum TypeKind {
    Enum,
    Function,
    Module,
    Const,
    Static,
    Struct,
    Union,
    Trait,
    Typedef,
    Foreign,
    Macro,
    Attr,
    Derive,
    TraitAlias,
}
