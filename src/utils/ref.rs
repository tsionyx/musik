use std::borrow::Cow;

pub fn to_static<T>(cow: Cow<'_, T>) -> Cow<'static, T>
where
    T: ToOwned + ?Sized,
    T::Owned: 'static,
{
    match cow {
        Cow::Borrowed(borrowed) => Cow::Owned(borrowed.to_owned()),
        Cow::Owned(owned) => Cow::Owned(owned),
    }
}
