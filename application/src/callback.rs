#[macro_export]
macro_rules! callback_body {
  ([$head: ident $(,$tail: ident)*] $body: tt) => {
    if let Some($head) = $head.upgrade() {
      callback_body!([$($tail),*] $body);
    }
  };

  ([] $body: tt) => {
    $body
  };
}

#[macro_export]
macro_rules! callback {
  ([$($ptr: ident),*] ($($args: ident),*) $body: tt) => {
    {
      #[allow(unused_parens)]
      let ($($ptr),*) = ($(Rc::downgrade(&$ptr)),*);
      move |$($args),*| {
        callback_body!([$($ptr),*] $body);
      }
    }
  };
}
