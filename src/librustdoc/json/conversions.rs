use std::convert::From;

use rustc_ast::ast;
use rustc_span::def_id;

use crate::clean;
use crate::doctree;
use crate::formats::item_type::ItemType;
use crate::json::types::*;

impl From<clean::Item> for Item {
    fn from(item: clean::Item) -> Self {
        let clean::Item {
            source,
            name,
            attrs,
            inner,
            visibility,
            def_id,
            stability: _,
            deprecation: _,
        } = item.clone();
        // TODO: dont clone
        Item {
            id: def_id.into(),
            crate_num: def_id.krate.as_u32(),
            name,
            source: source.into(),
            visibility: visibility.into(),
            docs: attrs.collapsed_doc_value(),
            attrs: attrs
                .other_attrs
                .iter()
                .map(rustc_ast_pretty::pprust::attribute_to_string)
                .collect(),
            kind: ItemType::from(&item).into(),
            inner: inner.into(),
            // attrs: unimplemented!(),
            // stability: stability.map(Into::into),
            // deprecation: deprecation.map(Into::into),
        }
    }
}

impl From<clean::Span> for Span {
    fn from(span: clean::Span) -> Self {
        let clean::Span { loline, locol, hiline, hicol, .. } = span;
        Span {
            filename: match span.filename {
                rustc_span::FileName::Real(name) => Some(match name {
                    rustc_span::RealFileName::Named(path) => path,
                    rustc_span::RealFileName::Devirtualized { local_path, virtual_name: _ } => {
                        local_path
                    }
                }),
                _ => None,
            },
            begin: (loline, locol),
            end: (hiline, hicol),
        }
    }
}

impl From<clean::Visibility> for Visibility {
    fn from(v: clean::Visibility) -> Self {
        use clean::Visibility::*;
        match v {
            Public => Visibility::Public,
            Inherited => Visibility::Inherited,
            Crate => Visibility::Crate,
            Restricted(did, path) => Visibility::Restricted(did.into(), path.into()),
        }
    }
}

impl From<clean::Path> for Path {
    fn from(path: clean::Path) -> Self {
        Path { global: path.global, segments: path.segments.into_iter().map(Into::into).collect() }
    }
}

impl From<clean::PathSegment> for PathSegment {
    fn from(segment: clean::PathSegment) -> Self {
        PathSegment { name: segment.name, args: segment.args.into() }
    }
}

impl From<clean::GenericArgs> for GenericArgs {
    fn from(args: clean::GenericArgs) -> Self {
        use clean::GenericArgs::*;
        match args {
            AngleBracketed { args, bindings } => GenericArgs::AngleBracketed {
                args: args.into_iter().map(Into::into).collect(),
                bindings: bindings.into_iter().map(Into::into).collect(),
            },
            Parenthesized { inputs, output } => GenericArgs::Parenthesized {
                inputs: inputs.into_iter().map(Into::into).collect(),
                output: output.map(Into::into),
            },
        }
    }
}

impl From<clean::GenericArg> for GenericArg {
    fn from(arg: clean::GenericArg) -> Self {
        use clean::GenericArg::*;
        match arg {
            Lifetime(l) => GenericArg::Lifetime(l.0),
            Type(t) => GenericArg::Type(t.into()),
            Const(c) => GenericArg::Const(c.into()),
        }
    }
}

impl From<clean::Constant> for Constant {
    fn from(constant: clean::Constant) -> Self {
        let clean::Constant { type_, expr, value, is_literal } = constant;
        Constant { type_: type_.into(), expr, value, is_literal }
    }
}

impl From<clean::TypeBinding> for TypeBinding {
    fn from(binding: clean::TypeBinding) -> Self {
        TypeBinding { name: binding.name, kind: binding.kind.into() }
    }
}

impl From<clean::TypeBindingKind> for TypeBindingKind {
    fn from(kind: clean::TypeBindingKind) -> Self {
        use clean::TypeBindingKind::*;
        match kind {
            Equality { ty } => TypeBindingKind::Equality(ty.into()),
            Constraint { bounds } => {
                TypeBindingKind::Constraint(bounds.into_iter().map(Into::into).collect())
            }
        }
    }
}

impl From<def_id::DefId> for Id {
    fn from(did: def_id::DefId) -> Self {
        Id(format!("{}:{}", did.krate.as_u32(), u32::from(did.index)))
    }
}

impl From<clean::ItemEnum> for ItemEnum {
    fn from(item: clean::ItemEnum) -> Self {
        use clean::ItemEnum::*;
        match item {
            ModuleItem(m) => ItemEnum::ModuleItem(m.into()),
            ExternCrateItem(c, a) => ItemEnum::ExternCrateItem(c, a),
            ImportItem(i) => ItemEnum::ImportItem(i.into()),
            StructItem(s) => ItemEnum::StructItem(s.into()),
            UnionItem(u) => ItemEnum::StructItem(u.into()),
            StructFieldItem(f) => ItemEnum::StructFieldItem(f.into()),
            EnumItem(e) => ItemEnum::EnumItem(e.into()),
            VariantItem(v) => ItemEnum::VariantItem(v.into()),
            FunctionItem(f) => ItemEnum::FunctionItem(f.into()),
            ForeignFunctionItem(f) => ItemEnum::ForeignFunctionItem(f.into()),
            TraitItem(t) => ItemEnum::TraitItem(t.into()),
            TraitAliasItem(t) => ItemEnum::TraitAliasItem(t.into()),
            MethodItem(m) => ItemEnum::MethodItem(m.into()),
            TyMethodItem(m) => ItemEnum::MethodItem(m.into()),
            ImplItem(i) => ItemEnum::ImplItem(i.into()),
            StaticItem(s) => ItemEnum::StaticItem(s.into()),
            ForeignStaticItem(s) => ItemEnum::ForeignStaticItem(s.into()),
            ForeignTypeItem => ItemEnum::ForeignTypeItem,
            TypedefItem(t, _) => ItemEnum::TypedefItem(t.into()),
            OpaqueTyItem(t, _) => ItemEnum::OpaqueTyItem(t.into()),
            ConstantItem(c) => ItemEnum::ConstantItem(c.into()),
            MacroItem(m) => ItemEnum::MacroItem(m.into()),
            ProcMacroItem(m) => ItemEnum::ProcMacroItem(m.into()),
            AssocConstItem(t, s) => ItemEnum::AssocConstItem(t.into(), s),
            AssocTypeItem(g, t) => {
                ItemEnum::AssocTypeItem(g.into_iter().map(Into::into).collect(), t.map(Into::into))
            }
            StrippedItem(inner) => ItemEnum::StrippedItem(Box::new((*inner).into())),
            _ => panic!("{:?} is not supported for JSON output", item),
        }
    }
}

impl From<clean::Module> for Module {
    fn from(module: clean::Module) -> Self {
        Module {
            is_crate: module.is_crate,
            items: module.items.into_iter().map(|i| i.def_id.into()).collect(),
        }
    }
}

impl From<clean::Struct> for Struct {
    fn from(struct_: clean::Struct) -> Self {
        let clean::Struct { struct_type, generics, fields, fields_stripped } = struct_;
        Struct {
            struct_type: struct_type.into(),
            generics: generics.into(),
            fields_stripped,
            fields: fields.into_iter().map(|i| i.def_id.into()).collect(),
        }
    }
}

impl From<clean::Union> for Struct {
    fn from(struct_: clean::Union) -> Self {
        let clean::Union { struct_type, generics, fields, fields_stripped } = struct_;
        Struct {
            struct_type: struct_type.into(),
            generics: generics.into(),
            fields_stripped,
            fields: fields.into_iter().map(|i| i.def_id.into()).collect(),
        }
    }
}

impl From<doctree::StructType> for StructType {
    fn from(struct_type: doctree::StructType) -> Self {
        use doctree::StructType::*;
        match struct_type {
            Plain => StructType::Plain,
            Tuple => StructType::Tuple,
            Unit => StructType::Unit,
        }
    }
}

impl From<clean::Function> for Function {
    fn from(function: clean::Function) -> Self {
        let clean::Function { decl, generics, header, all_types: _, ret_types: _ } = function;
        Function { decl: decl.into(), generics: generics.into(), header: header.into() }
    }
}

impl From<rustc_hir::FnHeader> for FnHeader {
    fn from(header: rustc_hir::FnHeader) -> Self {
        FnHeader {
            is_unsafe: header.unsafety == rustc_hir::Unsafety::Unsafe,
            is_const: header.constness == rustc_hir::Constness::Const,
            is_async: header.asyncness == rustc_hir::IsAsync::Async,
            abi: header.abi.to_string(),
        }
    }
}

impl From<clean::Generics> for Generics {
    fn from(generics: clean::Generics) -> Self {
        Generics {
            params: generics.params.into_iter().map(Into::into).collect(),
            where_predicates: generics.where_predicates.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<clean::GenericParamDef> for GenericParamDef {
    fn from(generic_param: clean::GenericParamDef) -> Self {
        GenericParamDef { name: generic_param.name, kind: generic_param.kind.into() }
    }
}

impl From<clean::GenericParamDefKind> for GenericParamDefKind {
    fn from(kind: clean::GenericParamDefKind) -> Self {
        use clean::GenericParamDefKind::*;
        match kind {
            Lifetime => GenericParamDefKind::Lifetime,
            Type { did: _, bounds, default, synthetic } => GenericParamDefKind::Type {
                bounds: bounds.into_iter().map(Into::into).collect(),
                default: default.map(Into::into),
                synthetic: synthetic.is_some(),
            },
            Const { did: _, ty } => GenericParamDefKind::Const(ty.into()),
        }
    }
}

impl From<clean::WherePredicate> for WherePredicate {
    fn from(predicate: clean::WherePredicate) -> Self {
        use clean::WherePredicate::*;
        match predicate {
            BoundPredicate { ty, bounds } => WherePredicate::BoundPredicate {
                ty: ty.into(),
                bounds: bounds.into_iter().map(Into::into).collect(),
            },
            RegionPredicate { lifetime, bounds } => WherePredicate::RegionPredicate {
                lifetime: lifetime.0,
                bounds: bounds.into_iter().map(Into::into).collect(),
            },
            EqPredicate { lhs, rhs } => {
                WherePredicate::EqPredicate { lhs: lhs.into(), rhs: rhs.into() }
            }
        }
    }
}

impl From<clean::GenericBound> for GenericBound {
    fn from(bound: clean::GenericBound) -> Self {
        use clean::GenericBound::*;
        match bound {
            TraitBound(clean::PolyTrait { trait_, generic_params }, modifier) => {
                GenericBound::TraitBound {
                    trait_: trait_.into(),
                    generic_params: generic_params.into_iter().map(Into::into).collect(),
                    modifier: modifier.into(),
                }
            }
            Outlives(lifetime) => GenericBound::Outlives(lifetime.0),
        }
    }
}

impl From<rustc_hir::TraitBoundModifier> for TraitBoundModifier {
    fn from(modifier: rustc_hir::TraitBoundModifier) -> Self {
        use rustc_hir::TraitBoundModifier::*;
        match modifier {
            None => TraitBoundModifier::None,
            Maybe => TraitBoundModifier::Maybe,
            MaybeConst => TraitBoundModifier::MaybeConst,
        }
    }
}

impl From<clean::PrimitiveType> for PrimitiveType {
    fn from(ty: clean::PrimitiveType) -> Self {
        use clean::PrimitiveType::*;
        match ty {
            Isize => PrimitiveType::Isize,
            I8 => PrimitiveType::I8,
            I16 => PrimitiveType::I16,
            I32 => PrimitiveType::I32,
            I64 => PrimitiveType::I64,
            I128 => PrimitiveType::I128,
            Usize => PrimitiveType::Usize,
            U8 => PrimitiveType::U8,
            U16 => PrimitiveType::U16,
            U32 => PrimitiveType::U32,
            U64 => PrimitiveType::U64,
            U128 => PrimitiveType::U128,
            F32 => PrimitiveType::F32,
            F64 => PrimitiveType::F64,
            Char => PrimitiveType::Char,
            Bool => PrimitiveType::Bool,
            Str => PrimitiveType::Str,
            Slice => PrimitiveType::Slice,
            Array => PrimitiveType::Array,
            Tuple => PrimitiveType::Tuple,
            Unit => PrimitiveType::Unit,
            RawPointer => PrimitiveType::RawPointer,
            Reference => PrimitiveType::Reference,
            Fn => PrimitiveType::Fn,
            Never => PrimitiveType::Never,
        }
    }
}

impl From<clean::Type> for Type {
    fn from(ty: clean::Type) -> Self {
        use clean::Type::*;
        match ty {
            ResolvedPath { path, param_names, did, is_generic } => Type::ResolvedPath {
                path: path.into(),
                param_names: param_names.map(|v| v.into_iter().map(Into::into).collect()),
                id: did.into(),
                is_generic,
            },
            Generic(s) => Type::Generic(s),
            Primitive(p) => Type::Primitive(p.into()),
            // TODO: check if there's a more idiomatic way of calling `into` on Box<T>
            BareFunction(f) => Type::BareFunction(Box::new((*f).into())),
            Tuple(t) => Type::Tuple(t.into_iter().map(Into::into).collect()),
            Slice(t) => Type::Slice(Box::new((*t).into())),
            Array(t, s) => Type::Array(Box::new((*t).into()), s),
            ImplTrait(g) => Type::ImplTrait(g.into_iter().map(Into::into).collect()),
            Never => Type::Never,
            Infer => Type::Infer,
            RawPointer(mutability, type_) => Type::RawPointer {
                mutable: mutability == ast::Mutability::Mut,
                type_: Box::new((*type_).into()),
            },
            BorrowedRef { lifetime, mutability, type_ } => Type::BorrowedRef {
                lifetime: lifetime.map(|l| l.0),
                mutable: mutability == ast::Mutability::Mut,
                type_: Box::new((*type_).into()),
            },
            QPath { name, self_type, trait_ } => Type::QPath {
                name,
                self_type: Box::new((*self_type).into()),
                trait_: Box::new((*trait_).into()),
            },
        }
    }
}

impl From<clean::BareFunctionDecl> for BareFunctionDecl {
    fn from(bare_decl: clean::BareFunctionDecl) -> Self {
        let clean::BareFunctionDecl { unsafety, generic_params, decl, abi } = bare_decl;
        BareFunctionDecl {
            is_unsafe: unsafety == rustc_hir::Unsafety::Unsafe,
            generic_params: generic_params.into_iter().map(Into::into).collect(),
            decl: decl.into(),
            abi: abi.to_string(),
        }
    }
}

impl From<clean::FnDecl> for FnDecl {
    fn from(decl: clean::FnDecl) -> Self {
        let clean::FnDecl { inputs, output, c_variadic, attrs: _ } = decl;
        FnDecl {
            inputs: inputs.values.into_iter().map(|arg| (arg.name, arg.type_.into())).collect(),
            output: match output {
                clean::FnRetTy::Return(t) => Some(t.into()),
                clean::FnRetTy::DefaultReturn => None,
            },
            c_variadic,
        }
    }
}

impl From<clean::Trait> for Trait {
    fn from(trait_: clean::Trait) -> Self {
        let clean::Trait { auto, unsafety, items, generics, bounds, is_auto: _ } = trait_;
        Trait {
            is_auto: auto,
            is_unsafe: unsafety == rustc_hir::Unsafety::Unsafe,
            items: items.into_iter().map(|i| i.def_id.into()).collect(),
            generics: generics.into(),
            bounds: bounds.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<clean::Impl> for Impl {
    fn from(impl_: clean::Impl) -> Self {
        let clean::Impl {
            unsafety,
            generics,
            provided_trait_methods,
            trait_,
            for_,
            items,
            polarity,
            synthetic,
            blanket_impl,
        } = impl_;
        Impl {
            is_unsafe: unsafety == rustc_hir::Unsafety::Unsafe,
            generics: generics.into(),
            provided_trait_methods: provided_trait_methods.into_iter().collect(),
            trait_: trait_.map(Into::into),
            for_: for_.into(),
            items: items.into_iter().map(|i| i.def_id.into()).collect(),
            negative: polarity == Some(clean::ImplPolarity::Negative),
            synthetic,
            blanket_impl: blanket_impl.map(Into::into),
        }
    }
}

impl From<clean::TyMethod> for Method {
    fn from(method: clean::TyMethod) -> Self {
        let clean::TyMethod { header, decl, generics, all_types: _, ret_types: _ } = method;
        Method {
            decl: decl.into(),
            generics: generics.into(),
            header: header.into(),
            has_body: false,
        }
    }
}

impl From<clean::Method> for Method {
    fn from(method: clean::Method) -> Self {
        let clean::Method { header, decl, generics, defaultness: _, all_types: _, ret_types: _ } =
            method;
        Method {
            decl: decl.into(),
            generics: generics.into(),
            header: header.into(),
            has_body: true,
        }
    }
}

impl From<clean::Enum> for Enum {
    fn from(enum_: clean::Enum) -> Self {
        let clean::Enum { variants, generics, variants_stripped } = enum_;
        Enum {
            generics: generics.into(),
            variants_stripped,
            variants: variants.into_iter().map(|i| i.def_id.into()).collect(),
        }
    }
}

impl From<clean::VariantStruct> for Struct {
    fn from(struct_: clean::VariantStruct) -> Self {
        let clean::VariantStruct { struct_type, fields, fields_stripped } = struct_;
        Struct {
            struct_type: struct_type.into(),
            generics: Default::default(),
            fields_stripped,
            fields: fields.into_iter().map(|i| i.def_id.into()).collect(),
        }
    }
}

impl From<clean::Variant> for Variant {
    fn from(variant: clean::Variant) -> Self {
        use clean::VariantKind::*;
        match variant.kind {
            CLike => Variant::Plain,
            Tuple(t) => Variant::Tuple(t.into_iter().map(Into::into).collect()),
            Struct(s) => Variant::Struct(s.into()),
        }
    }
}

impl From<clean::Import> for Import {
    fn from(import: clean::Import) -> Self {
        use clean::Import::*;
        match import {
            Simple(s, i) => Import::Simple(s, i.into()),
            Glob(i) => Import::Glob(i.into()),
        }
    }
}

impl From<clean::ImportSource> for ImportSource {
    fn from(source: clean::ImportSource) -> Self {
        ImportSource { path: source.path.into(), id: source.did.map(Into::into) }
    }
}

impl From<clean::Macro> for Macro {
    fn from(mac: clean::Macro) -> Self {
        Macro { source: mac.source, imported_from: mac.imported_from }
    }
}

impl From<clean::ProcMacro> for ProcMacro {
    fn from(mac: clean::ProcMacro) -> Self {
        ProcMacro { kind: mac.kind.into(), helpers: mac.helpers }
    }
}

impl From<rustc_span::hygiene::MacroKind> for MacroKind {
    fn from(kind: rustc_span::hygiene::MacroKind) -> Self {
        use rustc_span::hygiene::MacroKind::*;
        match kind {
            Bang => MacroKind::Bang,
            Attr => MacroKind::Attr,
            Derive => MacroKind::Derive,
        }
    }
}

impl From<clean::Typedef> for Typedef {
    fn from(typedef: clean::Typedef) -> Self {
        let clean::Typedef { type_, generics, item_type: _ } = typedef;
        Typedef { type_: type_.into(), generics: generics.into() }
    }
}

impl From<clean::OpaqueTy> for OpaqueTy {
    fn from(opaque: clean::OpaqueTy) -> Self {
        OpaqueTy {
            bounds: opaque.bounds.into_iter().map(Into::into).collect(),
            generics: opaque.generics.into(),
        }
    }
}

impl From<clean::Static> for Static {
    fn from(stat: clean::Static) -> Self {
        Static {
            type_: stat.type_.into(),
            mutable: stat.mutability == ast::Mutability::Mut,
            expr: stat.expr,
        }
    }
}

impl From<clean::TraitAlias> for TraitAlias {
    fn from(alias: clean::TraitAlias) -> Self {
        TraitAlias {
            generics: alias.generics.into(),
            bounds: alias.bounds.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<ItemType> for ItemKind {
    fn from(kind: ItemType) -> Self {
        use ItemType::*;
        match kind {
            Module => ItemKind::Module,
            ExternCrate => ItemKind::ExternCrate,
            Import => ItemKind::Import,
            Struct => ItemKind::Struct,
            Union => ItemKind::Union,
            Enum => ItemKind::Enum,
            Function => ItemKind::Function,
            Typedef => ItemKind::Typedef,
            OpaqueTy => ItemKind::OpaqueTy,
            Static => ItemKind::Static,
            Constant => ItemKind::Constant,
            Trait => ItemKind::Trait,
            Impl => ItemKind::Impl,
            TyMethod | Method => ItemKind::Method,
            StructField => ItemKind::StructField,
            Variant => ItemKind::Variant,
            Macro => ItemKind::Macro,
            Primitive => ItemKind::Primitive,
            AssocConst => ItemKind::AssocConst,
            AssocType => ItemKind::AssocType,
            ForeignType => ItemKind::ForeignType,
            Keyword => ItemKind::Keyword,
            TraitAlias => ItemKind::TraitAlias,
            ProcAttribute => ItemKind::ProcAttribute,
            ProcDerive => ItemKind::ProcDerive,
        }
    }
}
