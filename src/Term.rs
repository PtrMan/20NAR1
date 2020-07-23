#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub enum Copula {
    SIM, // <-> similarity
    INH, // --> inheritance
    PREDIMPL, // =/> predictive implication
    IMPL, // ==>
}

#[derive(PartialEq, Eq, Hash/*, Clone*/)]
pub enum Term {
    Cop(Copula, Box<Term>, Box<Term>),
    Name(String),
    Seq(Vec<Box<Term>>), // sequence
    SetInt(Vec<Box<Term>>),
    SetExt(Vec<Box<Term>>),
    DepVar(String), // #
    IndepVar(String), // $
    Conj(Vec<Box<Term>>), // &&
}

impl Clone for Term {
    fn clone(&self) -> Term {
        match &*self {
            Term::Cop(copula, subj, pred) => {
                Term::Cop(*copula, subj.clone(), pred.clone())
            }
            Term::Name(name) => Term::Name(name.clone()),
            Term::Seq(seq) => {
                let mut arr = vec![];
                for i in seq {
                    arr.push(i.clone());
                }
                Term::Seq(arr)
            },
            Term::SetInt(set) => {
                let mut arr = vec![];
                for i in set {
                    arr.push(i.clone());
                }
                Term::SetInt(arr)
            }
            Term::SetExt(set) => {
                let mut arr = vec![];
                for i in set {
                    arr.push(i.clone());
                }
                Term::SetExt(arr)
            },
            Term::DepVar(name) => {
                Term::DepVar(name.clone())
            },
            Term::IndepVar(name) => {
                Term::IndepVar(name.clone())
            },
            Term::Conj(elements) => {
                let mut arr = vec![];
                for i in elements {
                    arr.push(i.clone());
                }
                Term::Conj(arr)
            },
        }
    }
}

// helper
fn retSubterms2(t:&Term, res:&mut Vec<Term>) {
    res.push(t.clone());
    
    match t {
        Term::Cop(_, subj, pred) => {
            retSubterms2(&subj, res);
            retSubterms2(&pred, res);
        }
        Term::Seq(seq) => {
            for i in seq {
                retSubterms2(&i, res);
            }
        },
        Term::SetInt(set) => {
            for i in set {
                retSubterms2(&i, res);
            }
        }
        Term::SetExt(set) => {
            for i in set {
                retSubterms2(&i, res);
            }
        },
        Term::Conj(elements) => {
            for i in elements {
                retSubterms2(&i, res);
            }
        },
        _=>{}, // no special handling necessary for "terminal" ones
    }
}

pub fn retSubterms(t:&Term) -> Vec<Term> {
    let mut res=vec![];
    retSubterms2(&t,&mut res);
    res
}

pub fn calcComplexity(t:&Term) -> u64 {
    match t {
        Term::Cop(_, subj, pred) => {
            1 + calcComplexity(subj) + calcComplexity(pred)
        }
        Term::Name(_) => 1,
        Term::Seq(seq) => {
            let mut c = 0;
            for i in seq {
                c+=calcComplexity(i);
            }
            c
        },
        Term::SetInt(set) => {
            let mut c = 0;
            for i in set {
                c+=calcComplexity(i);
            }
            c
        }
        Term::SetExt(set) => {
            let mut c = 0;
            for i in set {
                c+=calcComplexity(i);
            }
            c
        },
        Term::DepVar(_) => {
            1
        },
        Term::IndepVar(_) => {
            1
        },
        Term::Conj(elements) => {
            let mut c = 0;
            for i in elements {
                c+=calcComplexity(i);
            }
            c
        },
    }
}