use crate::Term::*;

// API inspired by sseehh's idea to use functions to build terms

/// create product
pub fn p2(a:&Term,b:&Term)->Term {
  Term::Prod(vec![Box::new(a.clone()), Box::new(b.clone())])
}
/// create product
pub fn p3(a:&Term,b:&Term,c:&Term)->Term {
  Term::Prod(vec![Box::new(a.clone()), Box::new(b.clone()), Box::new(c.clone())])
}

/// create product with subterms
pub fn p(ts:&Vec<Term>)->Term {
  Term::Prod(ts.iter().map(|v| Box::new((*v).clone())).collect())
}

/// create statement
pub fn s(copula:Copula, subj:&Term,pred:&Term)->Term {
  Term::Stmt(copula, Box::new(subj.clone()), Box::new(pred.clone()))
}

/// create sequence
pub fn seq(ts:&Vec<Term>)->Term {
  Term::Seq(ts.iter().map(|v| Box::new((*v).clone())).collect())
}

/// create conjunction
pub fn conj(ts:&Vec<Term>)->Term {
  Term::Conj(ts.iter().map(|v| Box::new((*v).clone())).collect())
}
