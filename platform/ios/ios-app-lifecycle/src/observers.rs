pub(crate) trait ObserverRemover<T> {
    fn remove(&self, observer: &T);
}

pub(crate) struct ObserverOwner<C, T, const N: usize>
where
    C: ObserverRemover<T>,
{
    remover: C,
    observers: [T; N],
}

impl<C, T, const N: usize> ObserverOwner<C, T, N>
where
    C: ObserverRemover<T>,
{
    pub(crate) const fn new(remover: C, observers: [T; N]) -> Self {
        Self { remover, observers }
    }
}

impl<C, T, const N: usize> Drop for ObserverOwner<C, T, N>
where
    C: ObserverRemover<T>,
{
    fn drop(&mut self) {
        for observer in &self.observers {
            self.remover.remove(observer);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    struct FakeCenter(Rc<RefCell<Vec<u8>>>);

    impl ObserverRemover<u8> for FakeCenter {
        fn remove(&self, observer: &u8) {
            self.0.borrow_mut().push(*observer);
        }
    }

    #[test]
    fn owner_removes_each_observer_once_in_registration_order() {
        let removed = Rc::new(RefCell::new(Vec::new()));
        let owner = ObserverOwner::new(FakeCenter(Rc::clone(&removed)), [2, 4, 6, 8, 10]);
        assert!(removed.borrow().is_empty());
        drop(owner);
        assert_eq!(*removed.borrow(), [2, 4, 6, 8, 10]);
    }
}
