#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub enum Copula {
    SIM, // <-> similarity
    INH, // --> inheritance
    PREDIMPL, // =/> predictive implication
    IMPL, // ==>
    EQUIV, // <=>
}

#[derive(PartialEq, Eq, Hash/*, Clone*/)]
pub enum Term {
    Stmt(Copula, Box<Term>, Box<Term>), // statement ex: <a --> b>
    Name(String),
    Seq(Vec<Box<Term>>), // sequence
    SetInt(Vec<Box<Term>>),
    SetExt(Vec<Box<Term>>),
    QVar(String), // ?
    DepVar(String), // #
    IndepVar(String), // $
    Conj(Vec<Box<Term>>), // &&
    Prod(Vec<Box<Term>>), // product
    Img(Box<Term>, usize, Vec<Box<Term>>),// image (rel /idx+1 others)
    IntInt(Vec<Box<Term>>), // | intensional intersection
    ExtInt(Vec<Box<Term>>), // &  extensional intersection
    Par(Vec<Box<Term>>), // &| parallel events
    Neg(Box<Term>), // negation
}

impl Clone for Term {
    fn clone(&self) -> Term {
        match &*self {
            Term::Stmt(copula, subj, pred) => {
                Term::Stmt(*copula, subj.clone(), pred.clone())
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
            Term::QVar(name) => {
                Term::QVar(name.clone())
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
            Term::Prod(elements) => {
                let mut arr = vec![];
                for i in elements {
                    arr.push(i.clone());
                }
                Term::Prod(arr)
            },
            Term::Img(rel,idx,others) => {
                let mut arr = vec![];
                for i in others {
                    arr.push(i.clone());
                }
                Term::Img(rel.clone(),*idx,arr)
            },
            Term::IntInt(set) => {
                let mut arr = vec![];
                for i in set {
                    arr.push(i.clone());
                }
                Term::IntInt(arr)
            },
            Term::ExtInt(set) => {
                let mut arr = vec![];
                for i in set {
                    arr.push(i.clone());
                }
                Term::ExtInt(arr)
            }
            Term::Par(elements) => {
                let mut arr = vec![];
                for i in elements {
                    arr.push(i.clone());
                }
                Term::Par(arr)
            },
            Term::Neg(term) => {
                Term::Neg(term.clone())
            },
        }
    }
}

// helper
fn retSubterms2(t:&Term, res:&mut Vec<Term>) {
    res.push(t.clone());
    
    match t {
        Term::Stmt(_, subj, pred) => {
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
        Term::Prod(elements) => {
            for i in elements {
                retSubterms2(&i, res);
            }
        },
        Term::Img(rel,_idx,elements) => {
            for i in elements {
                retSubterms2(&i, res);
            }
            retSubterms2(&rel, res);
        },
        Term::IntInt(set) => {
            for i in set {
                retSubterms2(&i, res);
            }
        },
        Term::ExtInt(set) => {
            for i in set {
                retSubterms2(&i, res);
            }
        },
        Term::Par(elements) => {
            for i in elements {
                retSubterms2(&i, res);
            }
        },
        Term::Neg(term) => {
            retSubterms2(&term, res);
        },
        _=>{}, // no special handling necessary for "terminal" ones
    }
}

pub fn retSubterms(t:&Term) -> Vec<Term> {
    let mut res=vec![];
    retSubterms2(&t,&mut res);
    res
}

pub fn retUniqueSubterms(t:&Term)->Vec<Term> {
    // slow version with checking array, hashmap may be faster in some cases(?)
    let mut res:Vec<Term> = vec![];
    for iTerm in &retSubterms(t) {
        let mut inRes = false;
        for iTerm2 in &res {
            if checkEqTerm(&iTerm, &iTerm2) {
                inRes = true;
                break;
            }
        }
        if !inRes {
            res.push(iTerm.clone());
        }
    };
    res    
}


pub fn calcComplexity(t:&Term) -> u64 {
    match t {
        Term::Stmt(_, subj, pred) => {
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
        Term::QVar(_) => {
            1
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
        Term::Prod(elements) => {
            let mut c = 0;
            for i in elements {
                c+=calcComplexity(i);
            }
            c
        },
        Term::Img(rel,_idx,elements) => {
            let mut c = 0;
            for i in elements {
                c+=calcComplexity(i);
            }
            c+=calcComplexity(rel);
            c
        },
        Term::IntInt(set) => {
            let mut c = 0;
            for i in set {
                c+=calcComplexity(i);
            }
            c
        },
        Term::ExtInt(set) => {
            let mut c = 0;
            for i in set {
                c+=calcComplexity(i);
            }
            c
        },
        Term::Par(elements) => {
            let mut c = 0;
            for i in elements {
                c+=calcComplexity(i);
            }
            c
        },
        Term::Neg(term) => {
            1 + calcComplexity(term)
        },
    }
}

pub fn convTermToStr(t:&Term) -> String {
    match t {
        Term::Stmt(Copula, subj, pred) => {
            let subjStr = convTermToStr(subj);
            let predStr = convTermToStr(pred);
            let copStr = match Copula {Copula::SIM=>{"<->"},Copula::INH=>{"-->"},Copula::PREDIMPL=>"=/>",Copula::IMPL=>{"==>"},Copula::EQUIV=>{"<=>"}};
            format!("<{} {} {}>", subjStr, copStr, predStr)
        }
        Term::Name(name) => name.to_string(),
        Term::Seq(seq) => {
            let mut inner = convTermToStr(&seq[0]);
            for i in 1..seq.len() {
                inner = format!("{} , {}", inner, convTermToStr(&seq[i]));
            }
            format!("( {} )", inner)
        },
        Term::SetInt(set) => {
            let mut inner = convTermToStr(&set[0]);
            for i in 1..set.len() {
                inner = format!("{} {}", inner, convTermToStr(&set[i]));
            }
            format!("[{}]", inner)
        },
        Term::SetExt(set) => {
            let mut inner = convTermToStr(&set[0]);
            for i in 1..set.len() {
                inner = format!("{} {}", inner, convTermToStr(&set[i]));
            }
            format!("{{{}}}", inner)
        },
        Term::QVar(name) => {
            format!("?{}", name)
        },
        Term::DepVar(name) => {
            format!("#{}", name)
        },
        Term::IndepVar(name) => {
            format!("${}", name)
        },
        Term::Conj(elements) => {
            let mut inner = convTermToStr(&elements[0]);
            for i in 1..elements.len() {
                inner = format!("{} && {}", inner, convTermToStr(&elements[i]));
            }
            format!("( {} )", inner)
        },
        Term::Prod(elements) => {
            let mut inner = convTermToStr(&elements[0]);
            for i in 1..elements.len() {
                inner = format!("{} * {}", inner, convTermToStr(&elements[i]));
            }
            format!("( {} )", inner)
        },
        Term::Img(rel,idx,elements) => {
            let mut inner = convTermToStr(&elements[0]);
            for i in 1..elements.len() {
                inner = format!("{} {}", inner, convTermToStr(&elements[i]));
            }
            format!("( {} /{} {} )", convTermToStr(&rel), idx+1, inner)
        },
        Term::IntInt(elements) => {
            let mut inner = convTermToStr(&elements[0]);
            for i in 1..elements.len() {
                inner = format!("{} | {}", inner, convTermToStr(&elements[i]));
            }
            format!("( {} )", inner)
        },
        Term::ExtInt(elements) => {
            let mut inner = convTermToStr(&elements[0]);
            for i in 1..elements.len() {
                inner = format!("{} & {}", inner, convTermToStr(&elements[i]));
            }
            format!("( {} )", inner)
        },
        Term::Par(seq) => {
            let mut inner = convTermToStr(&seq[0]);
            for i in 1..seq.len() {
                inner = format!("{} ; {}", inner, convTermToStr(&seq[i]));
            }
            format!("( {} )", inner)
        },
        Term::Neg(term) => {
            let inner = convTermToStr(term);
            format!("(! {} )", inner)
        },
    }
}


pub fn checkEqTerm(a:&Term, b:&Term) -> bool {
    match a {
        Term::Stmt(copulaa, subja, preda) => {
            match b {
                Term::Stmt(copulab, subjb, predb) => copulaa == copulab && checkEqTerm(&subja, &subjb) && checkEqTerm(&preda, &predb),
                _ => false
            }
        }
        Term::Name(namea) => {
            match b {
                Term::Name(nameb) => namea == nameb,
                _ => false
            }
        },
        Term::Seq(seqa) => {
            match b {
                Term::Seq(seqb) => {
                    if seqa.len() == seqb.len() {
                        for idx in 0..seqa.len() {
                            if !checkEqTerm(&seqa[idx], &seqb[idx]) {return false};
                        }
                        true
                    }
                    else {false}
                },
                _ => false
            }
        },
        Term::SetInt(seta) => {
            match b {
                Term::SetInt(setb) => {
                    if seta.len() == setb.len() {
                        for idx in 0..seta.len() {
                            if !checkEqTerm(&seta[idx], &setb[idx]) {return false};
                        }
                        true
                    }
                    else {false}
                },
                _ => false
            }
        },
        Term::SetExt(seta) => {
            match b {
                Term::SetExt(setb) => {
                    if seta.len() == setb.len() {
                        for idx in 0..seta.len() {
                            if !checkEqTerm(&seta[idx], &setb[idx]) {return false};
                        }
                        true
                    }
                    else {false}
                },
                _ => false
            }
        },
        Term::QVar(namea) => {
            match b {
                Term::QVar(nameb) => namea == nameb,
                _ => false
            }
        },
        Term::DepVar(namea) => {
            match b {
                Term::DepVar(nameb) => namea == nameb,
                _ => false
            }
        },
        Term::IndepVar(namea) => {
            match b {
                Term::IndepVar(nameb) => namea == nameb,
                _ => false
            }
        },
        Term::Conj(elementsa) => {
            match b {
                Term::Conj(elementsb) => {
                    if elementsa.len() == elementsb.len() {
                        for idx in 0..elementsa.len() {
                            if !checkEqTerm(&elementsa[idx], &elementsb[idx]) {return false};
                        }
                        true
                    }
                    else {false}
                },
                _ => false
            }
        },
        Term::Prod(elementsa) => {
            match b {
                Term::Prod(elementsb) => {
                    if elementsa.len() == elementsb.len() {
                        for idx in 0..elementsa.len() {
                            if !checkEqTerm(&elementsa[idx], &elementsb[idx]) {return false};
                        }
                        true
                    }
                    else {false}
                },
                _ => false
            }
        },
        Term::Img(rela,idxa,elementsa) => {
            match b {
                Term::Img(relb,idxb,elementsb) if idxa==idxb => {
                    if !checkEqTerm(&rela, &relb) {return false};

                    if elementsa.len() == elementsb.len() {
                        for idx in 0..elementsa.len() {
                            if !checkEqTerm(&elementsa[idx], &elementsb[idx]) {return false};
                        }
                        true
                    }
                    else {false}
                },
                _ => false
            }
        },
        Term::IntInt(seta) => {
            match b {
                Term::IntInt(setb) => {
                    if seta.len() == setb.len() {
                        for idx in 0..seta.len() {
                            if !checkEqTerm(&seta[idx], &setb[idx]) {return false};
                        }
                        true
                    }
                    else {false}
                },
                _ => false
            }
        },
        Term::ExtInt(seta) => {
            match b {
                Term::ExtInt(setb) => {
                    if seta.len() == setb.len() {
                        for idx in 0..seta.len() {
                            if !checkEqTerm(&seta[idx], &setb[idx]) {return false};
                        }
                        true
                    }
                    else {false}
                },
                _ => false
            }
        },
        Term::Par(elementsa) => {
            match b {
                Term::Par(elementsb) => {
                    if elementsa.len() == elementsb.len() {
                        for idx in 0..elementsa.len() {
                            if !checkEqTerm(&elementsa[idx], &elementsb[idx]) {return false};
                        }
                        true
                    }
                    else {false}
                },
                _ => false
            }
        },
        Term::Neg(terma) => {
            match b {
                Term::Neg(termb) => checkEqTerm(&terma, &termb),
                _ => false
            }
        },
    }
}
