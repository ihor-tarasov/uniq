pub type VMError = Box<Box<str>>;
pub type VMResult<T = ()> = Result<T, VMError>;

pub(crate) fn vm_error<T>(message: String) -> VMResult<T> {
    Err(VMError::new(message.into_boxed_str()))
}
