#[allow(missing_docs)]
#[macro_export(local_inner_macros)]
macro_rules! scalar_float_ops {
    (
        $name:ident,
        $ops:expr
    ) => {
        scalar_float_ops!($name, $ops, f32);
    };
    (
        $name:ident,
        $ops:expr,
        $elem:ty
    ) => {
        #[derive(new)]
        struct $name<const D: usize> {
            desc: ScalarOperationDescription<$elem>,
        }

        impl<const D: usize, B: FusionBackend> Operation<B> for $name<D> {
            fn execute(self: Box<Self>, handles: &mut HandleContainer<B>) {
                let lhs = handles.get_float_tensor::<D>(&self.desc.lhs);
                let output = $ops(lhs, burn_tensor::ElementConversion::elem(self.desc.rhs));

                handles.register_float_tensor(&self.desc.out.id, output);
            }
        }
    };
    (
        $name:ident,
        $ops:expr,
        $elem:ty,
        noconvert
    ) => {
        #[derive(new)]
        struct $name<const D: usize> {
            desc: ScalarOperationDescription<$elem>,
        }

        impl<const D: usize, B: FusionBackend> Operation<B> for $name<D> {
            fn execute(self: Box<Self>, handles: &mut HandleContainer<B>) {
                let lhs = handles.get_float_tensor::<D>(&self.desc.lhs);
                let output = $ops(lhs, self.desc.rhs);

                handles.register_float_tensor(&self.desc.out.id, output);
            }
        }
    };
}

#[allow(missing_docs)]
#[macro_export(local_inner_macros)]
macro_rules! scalar_float2int_ops {
    (
        $name:ident,
        $ops:expr,
        $elem:ty
    ) => {
        #[derive(new)]
        struct $name<const D: usize> {
            desc: ScalarOperationDescription<$elem>,
        }

        impl<const D: usize, B: FusionBackend> Operation<B> for $name<D> {
            fn execute(self: Box<Self>, handles: &mut HandleContainer<B>) {
                let lhs = handles.get_float_tensor::<D>(&self.desc.lhs);
                let output = $ops(lhs, self.desc.rhs.clone());

                handles.register_int_tensor(&self.desc.out.id, output);
            }
        }
    };
}

#[allow(missing_docs)]
#[macro_export(local_inner_macros)]
macro_rules! unary_float_ops {
    (
        $name:ident,
        $ops:expr
    ) => {
        #[derive(new)]
        struct $name<const D: usize> {
            desc: UnaryOperationDescription,
        }

        impl<const D: usize, B: FusionBackend> Operation<B> for $name<D> {
            fn execute(self: Box<Self>, handles: &mut HandleContainer<B>) {
                let input = handles.get_float_tensor::<D>(&self.desc.input);
                let output = $ops(input);

                handles.register_float_tensor(&self.desc.out.id, output);
            }
        }
    };
}

#[allow(missing_docs)]
#[macro_export(local_inner_macros)]
macro_rules! unary_int_ops {
    (
        $name:ident,
        $ops:expr
    ) => {
        #[derive(new)]
        struct $name<const D: usize> {
            desc: UnaryOperationDescription,
        }

        impl<const D: usize, B: FusionBackend> Operation<B> for $name<D> {
            fn execute(self: Box<Self>, handles: &mut HandleContainer<B>) {
                let input = handles.get_int_tensor::<D>(&self.desc.input);
                let output = $ops(input);

                handles.register_int_tensor(&self.desc.out.id, output);
            }
        }
    };
}

#[allow(missing_docs)]
#[macro_export(local_inner_macros)]
macro_rules! scalar_float_cmp_ops {
    (
        $name:ident,
        $ops:expr
    ) => {
        #[derive(new)]
        struct $name<const D: usize> {
            desc: ScalarOperationDescription<f32>,
        }

        impl<const D: usize, B: FusionBackend> Operation<B> for $name<D> {
            fn execute(self: Box<Self>, handles: &mut HandleContainer<B>) {
                let lhs = handles.get_float_tensor::<D>(&self.desc.lhs);
                let output = $ops(lhs, burn_tensor::ElementConversion::elem(self.desc.rhs));

                handles.register_bool_tensor(&self.desc.out.id, output);
            }
        }
    };
}

#[allow(missing_docs)]
#[macro_export(local_inner_macros)]
macro_rules! scalar_int_cmp_ops {
    (
        $name:ident,
        $ops:expr
    ) => {
        #[derive(new)]
        struct $name<const D: usize> {
            desc: ScalarOperationDescription<i32>,
        }

        impl<const D: usize, B: FusionBackend> Operation<B> for $name<D> {
            fn execute(self: Box<Self>, handles: &mut HandleContainer<B>) {
                let lhs = handles.get_int_tensor::<D>(&self.desc.lhs);
                let output = $ops(lhs, burn_tensor::ElementConversion::elem(self.desc.rhs));

                handles.register_bool_tensor(&self.desc.out.id, output);
            }
        }
    };
}

#[allow(missing_docs)]
#[macro_export(local_inner_macros)]
macro_rules! scalar_int_ops {
    (
        $name:ident,
        $ops:expr
    ) => {
        scalar_int_ops!($name, $ops, i32);
    };
    (
        $name:ident,
        $ops:expr,
        $elem:ty
    ) => {
        #[derive(new)]
        struct $name<const D: usize> {
            desc: ScalarOperationDescription<$elem>,
        }

        impl<const D: usize, B: FusionBackend> Operation<B> for $name<D> {
            fn execute(self: Box<Self>, handles: &mut HandleContainer<B>) {
                let lhs = handles.get_int_tensor::<D>(&self.desc.lhs);
                let output = $ops(lhs, burn_tensor::ElementConversion::elem(self.desc.rhs));

                handles.register_int_tensor(&self.desc.out.id, output);
            }
        }
    };
    (
        $name:ident,
        $ops:expr,
        $elem:ty,
        noconvert
    ) => {
        #[derive(new)]
        struct $name<const D: usize> {
            desc: ScalarOperationDescription<$elem>,
        }

        impl<const D: usize, B: FusionBackend> Operation<B> for $name<D> {
            fn execute(self: Box<Self>, handles: &mut HandleContainer<B>) {
                let lhs = handles.get_int_tensor::<D>(&self.desc.lhs);
                let output = $ops(lhs, self.desc.rhs);

                handles.register_int_tensor(&self.desc.out.id, output);
            }
        }
    };
}
