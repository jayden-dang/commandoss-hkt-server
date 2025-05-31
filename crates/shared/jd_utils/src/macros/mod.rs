pub mod impl_from;

#[macro_export]
macro_rules! ensure {
  ($cond:expr, $err:expr) => {
    if !$cond {
      return Err($err);
    }
  };
}

#[macro_export]
macro_rules! bail {
  ($err:expr) => {
    return Err($err);
  };
}
