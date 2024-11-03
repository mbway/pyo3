#![cfg(all(Py_3_12, feature = "macros"))]

use std::any::TypeId;

use pyo3::impl_::pycell::PyVariableClassObject;
use pyo3::impl_::pyclass::PyClassImpl;
use pyo3::py_run;
use pyo3::types::{PyDict, PyTuple};
use pyo3::{prelude::*, types::PyType};

#[cfg(feature = "extend-opaque")]
use pyo3::types::PyInt;

#[path = "../src/tests/common.rs"]
mod common;

fn uses_variable_layout<T: PyClassImpl>() -> bool {
    TypeId::of::<T::Layout>() == TypeId::of::<PyVariableClassObject<T>>()
}

#[test]
#[cfg(not(feature = "extend-opaque"))]
#[should_panic(
    expected = "Cannot create pyclass MyClass because the layout of the base type does not match. \
    You may need to enable the `extend-opaque` feature."
)]
fn extending_opaque_without_feature_enabled() {
    #[pyclass(extends=PyType)]
    #[derive(Default)]
    struct MyClass {}

    #[pymethods]
    impl MyClass {
        #[pyo3(signature = (*_args, **_kwargs))]
        fn __init__(
            _slf: Bound<'_, MyClass>,
            _args: Bound<'_, PyTuple>,
            _kwargs: Option<Bound<'_, PyDict>>,
        ) {
        }
    }

    assert!(!uses_variable_layout::<MyClass>());

    Python::with_gil(|py| {
        let ty = py.get_type::<MyClass>();
        // panics when used
        py_run!(py, ty, "x = ty('X', (), {})");
    });
}

#[test]
#[cfg(feature = "extend-opaque")]
fn class_with_object_field() {
    #[pyclass(extends=PyType)]
    #[derive(Default)]
    struct ClassWithObjectField {
        #[pyo3(get, set)]
        value: Option<PyObject>,
    }

    #[pymethods]
    impl ClassWithObjectField {
        #[pyo3(signature = (*_args, **_kwargs))]
        fn __init__(
            _slf: Bound<'_, ClassWithObjectField>,
            _args: Bound<'_, PyTuple>,
            _kwargs: Option<Bound<'_, PyDict>>,
        ) {
        }
    }

    Python::with_gil(|py| {
        let ty = py.get_type::<ClassWithObjectField>();
        assert!(uses_variable_layout::<ClassWithObjectField>());
        py_run!(
            py,
            ty,
            "x = ty('X', (), {}); x.value = 5; assert x.value == 5"
        );
        py_run!(
            py,
            ty,
            "x = ty('X', (), {}); x.value = None; assert x.value == None"
        );

        let obj = Bound::new(py, ClassWithObjectField { value: None }).unwrap();
        py_run!(py, obj, "obj.value = 5");
        let obj_ref = obj.borrow();
        let Some(value) = &obj_ref.value else {
            panic!("obj_ref.value is None");
        };
        assert_eq!(*value.downcast_bound::<PyInt>(py).unwrap(), 5);
    });
}
