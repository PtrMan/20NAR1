use Term::*;

// sseehh API - create product
pub fn p2(a:&Term,b:&Term)->Term {
  Term::Prod(vec![Box::new(a.clone()), Box::new(b.clone())])
}

// sseehh API - create statement
pub fn s(copula:Copula, subj:&Term,pred:&Term)->Term {
  Term::Stmt(copula, Box::new(subj.clone()), Box::new(pred.clone()))
}

// sseehh API
pub fn seq(ts:&Vec<Term>)->Term {
  Term::Seq(ts.iter().map(|v| Box::new((*v).clone())).collect())
}

// sseehh API
pub fn conj(ts:&Vec<Term>)->Term {
  Term::Conj(ts.iter().map(|v| Box::new((*v).clone())).collect())
}
