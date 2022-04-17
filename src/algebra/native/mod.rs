use super::*;


impl<T> ScalarMath<T> for T
where
T: FloatT,
{
    fn clip(s: T, min_thresh: T, max_thresh: T, min_new: T, max_new: T) -> T
    {
        if s < min_thresh {min_new}
        else if s > max_thresh {max_new}
        else{s.clone()}
    }
}

impl<T> VectorMath<T> for [T]
where
    T: FloatT,
{

    fn copy_from(&mut self, src: &[T]){
        self.copy_from_slice(src);
    }

    fn translate(&mut self, c: T) {
        self.iter_mut().for_each(|x| *x += c);
    }

    fn scale(&mut self, c: T) {
        self.iter_mut().for_each(|x| *x *= c);
    }

    fn reciprocal(&mut self) {
        self.iter_mut().for_each(|x| *x = T::recip(*x));
    }

    fn sqrt(&mut self) {
        self.iter_mut().for_each(|x| *x = T::sqrt(*x));
    }

    fn rsqrt(&mut self) {
        self.iter_mut().for_each(|x| *x = T::recip(T::sqrt(*x)));
    }

    fn negate(&mut self) {
        self.iter_mut().for_each(|x| *x = -(*x));
    }

    fn hadamard(&mut self, y: &[T]) {
        self.iter_mut().zip(y).for_each(|(x,y)| *x = (*x)*(*y));
    }

    fn clip(& mut self, min_thresh: T, max_thresh: T, min_new: T, max_new: T){
        self.iter_mut().for_each(|x| {
            *x = T::clip(*x, min_thresh, max_thresh, min_new, max_new)
            }
        );
    }

    fn dot(&self, y: &[T]) -> T {
        self.iter().zip(y).fold(T::zero(),|acc, (&x, &y)| acc + x * y)
    }

    fn sumsq(&self) -> T {
        self.dot(self)
    }

    // 2-norm
    fn norm(&self) -> T {
        T::sqrt(self.sumsq())
    }

    //scaled norm of elementwise produce self.*v
    fn norm_scaled(&self, v: &[T]) -> T {

        assert_eq!(self.len(),v.len());
        let total = self.iter().zip(v).
        fold(T::zero(),|acc, (&x, &y)|{
            let prod = x * y;
            acc + prod * prod
        });
        T::sqrt(total)

    }

    // Returns infinity norm, ignoring NaNs
    fn norm_inf(&self) -> T {
        let mut out = T::zero();
        for v in self.iter().map(|v| v.abs()) {
            out = if v > out { v } else { out };
        }
        out
    }

    // Returns one norm
    fn norm_one(&self) -> T {
        self.iter().fold(T::zero(), |acc,v| acc + v.abs())
    }

    fn minimum(&self) -> T {
        self.iter().fold(T::infinity(), |r, &s| T::min(r, s))
    }

    fn maximum(&self) -> T {
        self.iter().fold(T::infinity(), |r, &s| T::max(r, s))
    }

    fn mean(&self) -> T {
        if self.len() == 0 {
            return T::zero();
        } else {
            let num = self.iter().fold(T::zero(), |r, &s| r + s);
            let den = T::from(self.len()).unwrap();
            return num / den;
        }
    }

    fn axpby(&mut self, a: T, x: &[T], b: T) {
        assert_eq!(self.len(), x.len());

        //handle b = 1 / 0 / -1 separately
        let yx = self.iter_mut().zip(x);
        if b == T::zero() {
            yx.for_each(|(y, x)| *y = a * (*x));
        } else if b == T::one() {
            yx.for_each(|(y, x)| *y = a * (*x) + (*y));
        } else if b == -T::one() {
            yx.for_each(|(y, x)| *y = a * (*x) - (*y));
        } else {
            yx.for_each(|(y, x)| *y = a * (*x) + b * (*y));
        }
    }

    fn waxpby(&mut self, a: T, x: &[T], b: T, y: &[T]) {
        assert_eq!(self.len(), x.len());
        assert_eq!(self.len(), y.len());

        let xy = x.iter().zip(y);

        for (w, (x, y)) in self.iter_mut().zip(xy) {
            *w = a * (*x) * b * (*y);
        }
    }
}



impl<T: FloatT> MatrixMath<T,[T]> for CscMatrix<T>
where
    T: FloatT
{

    //matrix properties
    fn nrows(&self) -> usize {self.m}
    fn ncols(&self) -> usize {self.n}
    fn is_square(&self) -> bool {self.m == self.n}

    //scalar mut operations
    fn scale(&mut self, c: T){
        self.nzval.scale(c);
    }

    fn col_norms(&self, norms: &mut [T]){
        norms.fill(T::zero());
        self.col_norms_no_reset(norms);
    }

    fn col_norms_no_reset(&self, norms: &mut [T]){

        assert_eq!(norms.len(),self.colptr.len()-1);

        for (i,v) in norms.iter_mut().enumerate(){
            for j in self.colptr[i]..self.colptr[i + 1]{
                let tmp = T::abs(self.nzval[j]);
                *v = T::max(*v,tmp);
            }
        }
    }

    fn col_norms_sym(&self, norms: &mut [T]){
        norms.fill(T::zero());
        self.col_norms_sym_no_reset(norms);
    }

    fn col_norms_sym_no_reset(&self, norms: &mut [T]){

        assert_eq!(norms.len(),self.colptr.len()-1);

        for i in 0 .. norms.len(){
            for j in self.colptr[i]..self.colptr[i + 1]{
                let tmp = T::abs(self.nzval[j]);
                let r   = self.rowval[j];
                norms[i] = T::max(norms[i],tmp);
                norms[r] = T::max(norms[r],tmp);
            }
        }
    }

    fn row_norms(&self, norms: &mut [T]){
        norms.fill(T::zero());
        self.row_norms_no_reset(norms);
    }

    fn row_norms_no_reset(&self, norms: &mut [T]){

        assert_eq!(self.rowval.len(),*self.colptr.last().unwrap());

        for (row,val) in self.rowval.iter().zip(self.nzval.iter()){
            norms[*row] = T::max(norms[*row],T::abs(*val));
        }
    }

    fn lmul_diag(&mut self, l: &[T]){

        let rows = &self.rowval;
        let vals = &mut self.nzval;

        for (val,row) in vals.iter_mut().zip(rows) {
            *val *= l[*row];
        }
    }

    fn rmul_diag(&mut self, r: &[T]){

        let colptr = &self.colptr;
        let vals = &mut self.nzval;

        assert_eq!(vals.len(),*colptr.last().unwrap());
        for i in 0..self.n {
            vals[colptr[i]..colptr[i+1]].scale(r[i]);
        }

    }

    fn lrmul_diag(&mut self, l: &[T], r: &[T]){

        assert_eq!(self.nzval.len(),*self.colptr.last().unwrap());

        for col in 0..self.n {
            let (first,last) = (self.colptr[col],self.colptr[col+1]);
            let  vals = &mut self.nzval[first..last];
            let rows = &self.rowval[first..last];

            for (val,row) in vals.iter_mut().zip(rows){
                *val *= l[*row] * r[col];
            }
        }
    }

    fn gemv(&self, y: &mut [T], trans: MatrixShape, x: &[T], a:T, b:T){

        match trans {
            MatrixShape::N => _csc_axpby_N(self, y, x, a, b),
            MatrixShape::T => _csc_axpby_T(self, y, x, a, b),
        }

    }
}


// sparse matrix-vector multiply, no transpose
#[allow(non_snake_case)]
fn _csc_axpby_N<T: FloatT>(A: &CscMatrix<T>, y: &mut [T], x: &[T], a:T, b:T)
where
    T:FloatT,
{

  //first do the b*y part
  if b == T::zero() {y.fill(T::zero())}
  else if b == T::one() {}
  else if b == -T::one() {y.negate()}
  else {y.scale(b)}

  // if a is zero, we're done
  if a == T::zero() {return}

  assert_eq!(A.nzval.len(),*A.colptr.last().unwrap());
  assert_eq!(x.len(),A.n);

  //y += A*x
  if a == T::one(){
      for j in 0..A.n {
          for i in A.colptr[j]..A.colptr[j+1]{
              y[A.rowval[i]] += A.nzval[i] * x[j];
          }
      }
  }
  else if a == -T::one(){
      for j in 0..A.n {
          for i in A.colptr[j]..A.colptr[j+1]{
              y[A.rowval[i]] -= A.nzval[i] * x[j];
          }
      }
  }
  else{
      for j in 0..A.n {
          for i in A.colptr[j]..A.colptr[j+1]{
              y[A.rowval[i]] += a * A.nzval[i] * x[j];
          }
      }
  }

}

// sparse matrix-vector multiply, transposed
#[allow(non_snake_case)]
fn _csc_axpby_T<T: FloatT>(A: &CscMatrix<T>, y: &mut [T], x: &[T], a:T, b:T){

    //first do the b*y part
    if b == T::zero() {y.fill(T::zero())}
    else if b == T::one() {}
    else if b == -T::one() {y.negate()}
    else {y.scale(b)}

    // if a is zero, we're done
    if a == T::zero() {return}

    assert_eq!(A.nzval.len(),*A.colptr.last().unwrap());
    assert_eq!(x.len(),A.n);

    //y += A*x
    if a == T::one(){
        for j in 0..A.n {
            for k in A.colptr[j]..A.colptr[j+1]{
                y[j] += A.nzval[k] * x[A.rowval[k]];
            }
        }
    }
    else if a == -T::one(){
        for j in 0..A.n {
            for k in A.colptr[j]..A.colptr[j+1]{
                y[j] -= A.nzval[k] * x[A.rowval[k]];
            }
        }
    }
    else{
        for j in 0..A.n {
            for k in A.colptr[j]..A.colptr[j+1]{
                y[j] += a * A.nzval[k] * x[A.rowval[k]];
            }
        }
    }

}
