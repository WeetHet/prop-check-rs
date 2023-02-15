use rand::Rng;
use rand::distributions::{Distribution, Standard};

type DynRand<T, A> = dyn FnMut(T) -> (A, T);
type BoxRand<T, A> = Box<DynRand<T, A>>;

pub trait PropRng: Rng + Clone
where
  Self: Sized + 'static, {
  fn int_value() -> BoxRand<Self, i32> {
    Box::new(move |mut rng| (rng.gen(), rng.clone()))
  }

  fn double_value() -> BoxRand<Self, f32> {
    Box::new(move |mut rng| (rng.gen(), rng.clone()))
  }

  fn map<A, B, F1, F2>(mut s: F1, mut f: F2) -> BoxRand<Self, B>
  where
    F1: FnMut(Self) -> (A, Self) + 'static,
    F2: FnMut(A) -> B + 'static, {
    Box::new(move |rng| {
      let (a, rng2) = s(rng);
      (f(a), rng2)
    })
  }

  fn map2<F1, F2, F3, A, B, C>(mut ra: F1, mut rb: F2, mut f: F3) -> BoxRand<Self, C>
  where
    F1: FnMut(Self) -> (A, Self) + 'static,
    F2: FnMut(Self) -> (B, Self) + 'static,
    F3: FnMut(A, B) -> C + 'static, {
    Box::new(move |rng| {
      let (a, r1) = ra(rng);
      let (b, r2) = rb(r1);
      (f(a, b), r2)
    })
  }

  fn both<F1, F2, A, B>(ra: F1, rb: F2) -> BoxRand<Self, (A, B)>
  where
    F1: FnMut(Self) -> (A, Self) + 'static,
    F2: FnMut(Self) -> (B, Self) + 'static, {
    Self::map2(ra, rb, |a, b| (a, b))
  }

  fn unit<A>(a: A) -> BoxRand<Self, A>
  where
    A: Clone + 'static, {
    Box::new(move |rng: Self| (a.clone(), rng))
  }

  fn sequence<A, F>(fs: Vec<F>) -> BoxRand<Self, Vec<A>>
  where
    A: Clone + 'static,
    F: FnMut(Self) -> (A, Self) + 'static, {
    let unit = Self::unit(Vec::<A>::new());

    fs.into_iter().fold(unit, |acc, e| {
      Self::map2(acc, e, |mut a, b| {
        a.push(b);
        a
      })
    })
  }

  fn rand_int_double() -> BoxRand<Self, (i32, f32)> {
    Self::both(Self::int_value(), Self::double_value())
  }

  fn rand_double_int() -> BoxRand<Self, (f32, i32)> {
    Self::both(Self::double_value(), Self::int_value())
  }

  fn flat_map<A, B, F, GF, BF>(mut f: F, mut g: GF) -> BoxRand<Self, B>
  where
    F: FnMut(Self) -> (A, Self) + 'static,
    BF: FnMut(Self) -> (B, Self) + 'static,
    GF: FnMut(A) -> BF + 'static, {
    Box::new(move |rng| {
      let (a, r1) = f(rng);
      (g(a))(r1)
    })
  }

  fn gen_unfold<T>(&self) -> (T, Self)
  where
    Standard: Distribution<T>, {
    let mut rng = self.clone();
    (rng.gen(), rng)
  }
}

impl<R: Rng + Clone + 'static> PropRng for R {}
