use crate::environment::{Environment, Identifier, Type};
use crate::traits::MatrixNumber;
use crate::WindowState;
use std::collections::HashMap;

pub fn insert_to_env<T: MatrixNumber>(
    env: &mut Environment<T>,
    identifier: Identifier,
    value: Type<T>,
    windows: &mut HashMap<Identifier, WindowState>,
) {
    env.insert(identifier.clone(), value);
    windows.insert(identifier, WindowState { is_open: true });
}
