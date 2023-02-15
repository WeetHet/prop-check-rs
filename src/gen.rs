use crate::rng::PropRng;
use crate::state::State;

use rand::distributions::{Standard, Distribution};
use rand::distributions::uniform::SampleUniform;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::iter::FromIterator;
use std::marker::PhantomData;
use std::rc::Rc;

/// Factory responsible for making Gens.<br/>
pub struct Gens<R: PropRng> {
  _phantom: PhantomData<R>,
}

impl<R: PropRng + 'static> Gens<R> {
  /// Makes a Gen that returns `()`.<br/>
  pub fn unit() -> Gen<R, ()> {
    Self::pure(())
  }

  /// Makes a Gen that returns a value.<br/>
  pub fn pure<B>(value: B) -> Gen<R, B>
  where
    B: Clone + 'static, {
    Gen::<R, B>::new(State::value(value))
  }

  /// Makes a Gen that returns a value from a function.<br/>
  pub fn pure_lazy<B, F>(f: F) -> Gen<R, B>
  where
    F: Fn() -> B + 'static,
    B: Clone + 'static, {
    Self::pure(()).map(move |_| f())
  }

  /// Makes a Gen that wraps the value of Gen into Option.<br/>
  pub fn some<B>(gen: Gen<R, B>) -> Gen<R, Option<B>>
  where
    B: Clone + 'static, {
    gen.map(Some)
  }

  /// Makes a Gen that returns Some or None based on the value of Gen.<br/>
  pub fn option<B>(gen: Gen<R, B>) -> Gen<R, Option<B>>
  where
    B: Debug + Clone + 'static, {
    Self::frequency([(1, Self::pure(None)), (9, Self::some(gen))])
  }

  /// Makes a Gen that returns Either based on two Gens.<br/>
  pub fn either<T, E>(gt: Gen<R, T>, ge: Gen<R, E>) -> Gen<R, Result<T, E>>
  where
    T: Clone + 'static,
    E: Clone + 'static, {
    Self::one_of([gt.map(Ok), ge.map(Err)])
  }

  /// Makes a Gen that produces values according to a specified ratio.<br/>
  pub fn frequency_values<B>(values: impl IntoIterator<Item = (u32, B)>) -> Gen<R, B>
  where
    B: Debug + Clone + 'static, {
    Self::frequency(values.into_iter().map(|(n, value)| (n, Gens::pure(value))))
  }

  /// Makes a Gen that produces a value based on the specified ratio and Gen.<br/>
  pub fn frequency<B>(values: impl IntoIterator<Item = (u32, Gen<R, B>)>) -> Gen<R, B>
  where
    B: Debug + Clone + 'static, {
    let filtered = values.into_iter().filter(|kv| kv.0 > 0);
    let (tree, total) = filtered.fold((BTreeMap::new(), 0), |(mut tree, total), (weight, value)| {
      let t = total + weight;
      tree.insert(t, value);
      (tree, t)
    });
    Self::choose(1, total).flat_map(move |n| tree.range(n..).next().unwrap().1.clone())
  }

  /// Makes a Gen whose elements are the values generated by the specified number of Gen.<br/>
  pub fn list_of_n<B>(n: usize, gen: Gen<R, B>) -> Gen<R, Vec<B>>
  where
    B: Clone + 'static, {
    let v: Vec<State<R, B>> = (0..n).map(move |_| gen.clone().sample).collect();
    Gen {
      sample: State::sequence(v),
    }
  }

  /// Makes a Gen that returns a single value of a certain type.<br/>
  pub fn one<T: Clone + 'static>() -> Gen<R, T>
  where Standard : Distribution<T>
   {
    Gen {
      sample: State::<R, T>::new(move |mut rng: R| (rng.gen(), rng.clone())),
    }
  }

  /// Makes a Gen that returns a value selected at random from a specified set of Gen.<br/>
  pub fn one_of<T: Clone + 'static>(values: impl IntoIterator<Item = Gen<R, T>>) -> Gen<R, T> {
    let vec = Vec::from_iter(values);
    let len = vec.len();
    Gen {
      sample: State::<R, usize>::new(move |mut rng: R| (rng.gen_range(0..len), rng.clone())),
    }
    .flat_map(move |idx| vec[idx].clone())
  }

  /// Makes a Gen that returns one randomly selected value from the specified set of values.<br/>
  pub fn one_of_values<T: Clone + 'static>(values: impl IntoIterator<Item = T>) -> Gen<R, T> {
    Self::one_of(values.into_iter().map(Gens::pure))
  }

  /// Makes a Gen that returns one randomly selected value from the specified maximum and minimum ranges of generic type.<br/>
  pub fn choose<T>(min: T, max: T) -> Gen<R, T>
  where
    T: SampleUniform + PartialOrd + Clone + 'static {
    Gen {
      sample: State::<R, T>::new(move |mut rng: R| (rng.gen_range(min.clone()..max.clone()), rng.clone())),
    }
  }
}

/// Generator that Makes values.<br/>
#[derive(Debug, Clone)]
pub struct Gen<T: PropRng + 'static, A: 'static> {
  sample: State<T, A>,
}

impl<T: PropRng, A: Clone + 'static> Gen<T, A> {
  /// Evaluates expressions held by the Gen and Makes values.<br/>  
  pub fn run(self, rng: T) -> (A, T) {
    self.sample.run(rng)
  }

  /// Generate a Gen by specifying a State.<br/>
  pub fn new<B>(b: State<T, B>) -> Gen<T, B> {
    Gen { sample: b }
  }

  /// Applies a function to Gen.<br/>
  pub fn map<B, F>(self, f: F) -> Gen<T, B>
  where
    F: Fn(A) -> B + 'static,
    B: Clone + 'static, 
    T: 'static {
    Self::new(self.sample.map(f))
  }

  /// Applies a function that takes the result of two Gen's as arguments.<br/>
  pub fn and_then<B, C, F>(self, g: Gen<T, B>, f: F) -> Gen<T, C>
  where
    F: Fn(A, B) -> C + 'static,
    A: Clone,
    B: Clone + 'static,
    C: Clone + 'static,
    T: 'static {
    Self::new(self.sample.and_then(g.sample).map(move |(a, b)| f(a, b)))
  }

  /// Applies a function to a Gen that takes the result of the Gen as an argument and returns the result.<br/>
  pub fn flat_map<B, F>(self, f: F) -> Gen<T, B>
  where
    F: Fn(A) -> Gen<T, B> + 'static,
    B: Clone + 'static,
    T: 'static {
    Self::new(self.sample.flat_map(move |a| f(a).sample))
  }
}

pub enum SGen<T: PropRng + 'static, A: 'static> {
  Sized(Rc<RefCell<dyn Fn(u32) -> Gen<T, A>>>),
  Unsized(Gen<T, A>),
}

impl<T: PropRng, A: Clone + 'static> Clone for SGen<T, A> {
  fn clone(&self) -> Self {
    match self {
      SGen::Sized(f) => SGen::Sized(f.clone()),
      SGen::Unsized(g) => SGen::Unsized(g.clone()),
    }
  }
}

impl<T: PropRng, A: Clone + 'static> SGen<T, A> {
  pub fn of_sized<F>(f: F) -> SGen<T, A>
  where
    F: Fn(u32) -> Gen<T, A> + 'static, {
    SGen::Sized(Rc::new(RefCell::new(f)))
  }

  pub fn of_unsized(gen: Gen<T, A>) -> SGen<T, A> {
    SGen::Unsized(gen)
  }

  pub fn run(&self, i: Option<u32>) -> Gen<T, A> {
    match self {
      SGen::Sized(f) => {
        let mf = f.borrow_mut();
        mf(i.unwrap())
      }
      SGen::Unsized(g) => g.clone(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::prop;
  use anyhow::Result;
use rand::thread_rng;

  use std::cell::RefCell;
  use std::collections::HashMap;
  use std::env;
  use std::rc::Rc;

  #[ctor::ctor]
  fn init() {
    env::set_var("RUST_LOG", "info");
    let _ = env_logger::builder().is_test(true).try_init();
  }

  pub mod laws {
    use rand::{rngs::StdRng, SeedableRng, thread_rng};

    use super::*;

    #[test]
    fn test_left_identity_law() -> Result<()> {
      let gen = Gens::choose(1, i32::MAX / 2).map(|e| (StdRng::seed_from_u64(e as u64), e));
      #[allow(clippy::redundant_closure)]
      let f = |x| Gens::pure(x);
      let laws_prop = prop::for_all_gen(gen, move |(s, n)| {
        Gens::pure(n).flat_map(f).run(s.clone()) == f(n).run(s)
      });
      prop::test_with_prop(laws_prop, 1, 100, thread_rng())
    }

    #[test]
    fn test_right_identity_law() -> Result<()> {
      let gen = Gens::choose(1, i32::MAX / 2).map(|e| (StdRng::seed_from_u64(e as u64), e));

      #[allow(clippy::redundant_closure)]
      let laws_prop = prop::for_all_gen(gen, move |(s, x)| {
        Gens::pure(x).flat_map(|y| Gens::pure(y)).run(s.clone()) == Gens::pure(x).run(s)
      });

      prop::test_with_prop(laws_prop, 1, 100, thread_rng())
    }

    #[test]
    fn test_associativity_law() -> Result<()> {
      let gen = Gens::choose(1, i32::MAX / 2).map(|e| (StdRng::seed_from_u64(e as u64), e));
      let f = |x| Gens::pure(x * 2);
      let g = |x| Gens::pure(x + 1);
      let laws_prop = prop::for_all_gen(gen, move |(s, x)| {
        Gens::pure(x).flat_map(f).flat_map(g).run(s.clone()) == f(x).flat_map(g).run(s)
      });
      prop::test_with_prop(laws_prop, 1, 100, thread_rng())
    }
  }

  #[test]
  fn test_frequency() -> Result<()> {
    let gens = [
      (1, Gens::choose(1, 10)),
      (1, Gens::choose(50, 100)),
      (1, Gens::choose(200, 300)),
    ];
    let gen = Gens::frequency(gens);
    let prop = prop::for_all_gen(gen, move |a| {
      log::info!("a: {}", a);
      #[allow(clippy::if_same_then_else)]
      #[allow(clippy::needless_bool)]
      if (1..=10).contains(&a) {
        true
      } else if (50..=100).contains(&a) {
        true
      } else if (200..=300).contains(&a) {
        true
      } else {
        false
      }
    });
    prop::test_with_prop(prop, 1, 100, thread_rng())
  }

  #[test]
  fn test_frequency_values() -> Result<()> {
    let result = Rc::new(RefCell::new(HashMap::new()));
    let cloned_map = result.clone();
    let gens = [(1, "a"), (1, "b"), (8, "c")];
    let gen = Gens::frequency_values(gens);
    let prop = prop::for_all_gen(gen, move |a| {
      let mut map = result.borrow_mut();
      let r = map.entry(a).or_insert_with(|| 0);
      *r += 1;
      true
    });
    let r = prop::test_with_prop(prop, 1, 100, thread_rng());
    let map = cloned_map.borrow();
    let a_count = map.get(&"a").unwrap();
    let b_count = map.get(&"b").unwrap();
    let c_count = map.get(&"c").unwrap();
    assert_eq!(*a_count + *b_count + *c_count, 100);
    println!("{cloned_map:?}");
    r
  }
}
