
use nom::{
  IResult,
  bytes::complete::{tag, take_while_m_n},
  combinator::map_res,
  //Err,
//  sequence::tuple,
//  take_while,
//  alt,
};
//use nom::character::is_alphanumeric;
//use nom::named;
//use nom::many_m_n;

use crate::Term::*;
use crate::NarSentence::EnumPunctation;
use crate::Tv::Tv;
use crate::TermApi::*;

// finds out if narsese has tv and returns TV if TV exists
// cuts away narsese of TV if TV is detected
pub fn parseNarseseRetTv(narsese:&mut String, tv:&mut Tv) {
  *tv = Tv{f:1.0,c:0.9}; // set TV to default
  
  if narsese.len() > 0 && narsese.chars().nth(narsese.len()-1).unwrap() == '}' {
    for revIdx in 1..narsese.len() { // scan for '{'
      let idx = narsese.len() - revIdx; // compute index from back
      
      if narsese.chars().nth(idx).unwrap() == '{' {
        let tvStr = &narsese[idx+1..narsese.len()-1];
        let splitted:Vec<&str> = tvStr.split(" ").collect();
        
        if splitted.len() == 2 { // must have two values
          // TODO< handle error better >
          let f = splitted[0].parse::<f64>().unwrap();
          let c = splitted[1].parse::<f64>().unwrap();
          *narsese = narsese[..idx].to_string(); // cut away
          *tv = Tv{f:f,c:c};
          return;
        }
        return;
      }
    }
  }
}

pub fn parseNarsese(narsese:&String) -> Option<(Term, Tv, EnumPunctation, bool)> {
  let mut narsese2:String = narsese.clone();

  let mut isEvent = false;
  if narsese2.ends_with(" :|:") {
    isEvent = true;
    narsese2 = narsese2[..narsese2.len()-4].to_string();
  }

  let mut tv = Tv{f:1.0,c:0.9};
  parseNarseseRetTv(&mut narsese2, &mut tv);
  //println!("{}", &narsese2);
  narsese2 = narsese2.trim_end().to_string();
  //println!("{}", &narsese2);

  if narsese2.len() == 0 {
    return None; // can't parse empty string!
  }
  
  let punctationChar = narsese2.chars().nth(narsese2.len()-1).unwrap();
  let narseseInner = narsese2[..narsese2.len()-1].to_string();
  
  let punctation = match punctationChar {
    '.' => {EnumPunctation::JUGEMENT},
    '!' => {EnumPunctation::GOAL},
    '?' => {EnumPunctation::QUESTION},
    _ => {return None;},
  };

  //println!("{}   {}", narseseInner, punctationChar);
  //println!("f = {}", tv.f);
  //println!("c = {}", tv.c);

  let parsed:IResult<&str, Term> = parseEntry(&narseseInner);

  match parsed {
    Ok((_str2, term)) => {
      Some((term, tv, punctation, isEvent))
    },
    Err(_) => {
      None
    }
  }
}





#[cfg(test)]
mod tests {
  use super::*;
  use crate::Term::convTermToStr;

  #[test]
  pub fn inhWithTv() {
    let narsese = "<a --> b>. {0.4 0.8}".to_string();
    let parseResOpt: Option<(Term, Tv, EnumPunctation, bool)> = parseNarsese(&narsese);
    assert_eq!(parseResOpt.is_some(), true);
    
    let (term, tv, punct, _isEvent) = parseResOpt.unwrap();
    assert_eq!(convTermToStr(&term), "<a --> b>");
    assert_eq!((tv.f - 0.4).abs() < 0.01, true);
    assert_eq!((tv.c - 0.8).abs() < 0.01, true);
    assert_eq!(punct, EnumPunctation::JUGEMENT);
  }

  #[test]
  pub fn inhWithoutTv() {
    let narsese = "<a --> b>.".to_string();
    let parseResOpt: Option<(Term, Tv, EnumPunctation, bool)> = parseNarsese(&narsese);
    assert_eq!(parseResOpt.is_some(), true);
    
    let (term, tv, punct, _isEvent) = parseResOpt.unwrap();
    assert_eq!(convTermToStr(&term), "<a --> b>");
    assert_eq!((tv.f - 1.0).abs() < 0.01, true);
    assert_eq!((tv.c - 0.9).abs() < 0.01, true);
    assert_eq!(punct, EnumPunctation::JUGEMENT);
  }

  #[test]
  pub fn predImpl() {
    let narsese = "<a =/> b>.".to_string();
    let parseResOpt: Option<(Term, Tv, EnumPunctation, bool)> = parseNarsese(&narsese);
    assert_eq!(parseResOpt.is_some(), true);
    
    let (term, tv, punct, _isEvent) = parseResOpt.unwrap();
    assert_eq!(convTermToStr(&term), "<a =/> b>");
    assert_eq!((tv.f - 1.0).abs() < 0.01, true);
    assert_eq!((tv.c - 0.9).abs() < 0.01, true);
    assert_eq!(punct, EnumPunctation::JUGEMENT);
  }

  #[test]
  pub fn equiv() {
    let narsese = "<a <=> b>.".to_string();
    let parseResOpt: Option<(Term, Tv, EnumPunctation, bool)> = parseNarsese(&narsese);
    assert_eq!(parseResOpt.is_some(), true);
    
    let (term, tv, punct, _isEvent) = parseResOpt.unwrap();
    assert_eq!(convTermToStr(&term), "<a <=> b>");
    assert_eq!((tv.f - 1.0).abs() < 0.01, true);
    assert_eq!((tv.c - 0.9).abs() < 0.01, true);
    assert_eq!(punct, EnumPunctation::JUGEMENT);
  }


  #[test]
  pub fn numeric() {
    let narsese = "<0 --> b>.".to_string();
    let parseResOpt: Option<(Term, Tv, EnumPunctation, bool)> = parseNarsese(&narsese);
    assert_eq!(parseResOpt.is_some(), true);
    
    let (term, tv, punct, _isEvent) = parseResOpt.unwrap();
    assert_eq!(convTermToStr(&term), "<0 --> b>");
    assert_eq!((tv.f - 1.0).abs() < 0.01, true);
    assert_eq!((tv.c - 0.9).abs() < 0.01, true);
    assert_eq!(punct, EnumPunctation::JUGEMENT);
  }

  #[test]
  pub fn qVar() {
    let narsese = "<?a --> b>.".to_string();
    let parseResOpt: Option<(Term, Tv, EnumPunctation, bool)> = parseNarsese(&narsese);
    assert_eq!(parseResOpt.is_some(), true);
    
    let (term, tv, punct, _isEvent) = parseResOpt.unwrap();
    assert_eq!(convTermToStr(&term), "<?a --> b>");
    assert_eq!((tv.f - 1.0).abs() < 0.01, true);
    assert_eq!((tv.c - 0.9).abs() < 0.01, true);
    assert_eq!(punct, EnumPunctation::JUGEMENT);
  }

  #[test]
  pub fn indepVar() {
    let narsese = "<$a --> b>.".to_string();
    let parseResOpt: Option<(Term, Tv, EnumPunctation, bool)> = parseNarsese(&narsese);
    assert_eq!(parseResOpt.is_some(), true);
    
    let (term, tv, punct, _isEvent) = parseResOpt.unwrap();
    assert_eq!(convTermToStr(&term), "<$a --> b>");
    assert_eq!((tv.f - 1.0).abs() < 0.01, true);
    assert_eq!((tv.c - 0.9).abs() < 0.01, true);
    assert_eq!(punct, EnumPunctation::JUGEMENT);
  }

  #[test]
  pub fn depVar() {
    let narsese = "<#a --> b>.".to_string();
    let parseResOpt: Option<(Term, Tv, EnumPunctation, bool)> = parseNarsese(&narsese);
    assert_eq!(parseResOpt.is_some(), true);
    
    let (term, tv, punct, _isEvent) = parseResOpt.unwrap();
    assert_eq!(convTermToStr(&term), "<#a --> b>");
    assert_eq!((tv.f - 1.0).abs() < 0.01, true);
    assert_eq!((tv.c - 0.9).abs() < 0.01, true);
    assert_eq!(punct, EnumPunctation::JUGEMENT);
  }


  #[test]
  pub fn setInt() {
    let narsese = "<a --> [b]>.".to_string();
    let parseResOpt: Option<(Term, Tv, EnumPunctation, bool)> = parseNarsese(&narsese);
    assert_eq!(parseResOpt.is_some(), true);
    
    let (term, tv, punct, _isEvent) = parseResOpt.unwrap();
    assert_eq!(convTermToStr(&term), "<a --> [b]>");
    assert_eq!((tv.f - 1.0).abs() < 0.01, true);
    assert_eq!((tv.c - 0.9).abs() < 0.01, true);
    assert_eq!(punct, EnumPunctation::JUGEMENT);
  }

  #[test]
  pub fn setExt() {
    let narsese = "<{a} --> b>.".to_string();
    let parseResOpt: Option<(Term, Tv, EnumPunctation, bool)> = parseNarsese(&narsese);
    assert_eq!(parseResOpt.is_some(), true);
    
    let (term, tv, punct, _isEvent) = parseResOpt.unwrap();
    assert_eq!(convTermToStr(&term), "<{a} --> b>");
    assert_eq!((tv.f - 1.0).abs() < 0.01, true);
    assert_eq!((tv.c - 0.9).abs() < 0.01, true);
    assert_eq!(punct, EnumPunctation::JUGEMENT);
  }

  #[test]
  pub fn setExtProd() {
    let narsese = "<{(a*c)} --> b>.".to_string();
    let parseResOpt: Option<(Term, Tv, EnumPunctation, bool)> = parseNarsese(&narsese);
    assert_eq!(parseResOpt.is_some(), true);
    
    let (term, tv, punct, _isEvent) = parseResOpt.unwrap();
    assert_eq!(convTermToStr(&term), "<{( a * c )} --> b>");
    assert_eq!((tv.f - 1.0).abs() < 0.01, true);
    assert_eq!((tv.c - 0.9).abs() < 0.01, true);
    assert_eq!(punct, EnumPunctation::JUGEMENT);
  }

  #[test]
  pub fn prod2() {
    let narsese = "<(a*c) --> x>.".to_string();
    let parseResOpt: Option<(Term, Tv, EnumPunctation, bool)> = parseNarsese(&narsese);
    assert_eq!(parseResOpt.is_some(), true);
    
    let (term, tv, punct, _isEvent) = parseResOpt.unwrap();
    assert_eq!(convTermToStr(&term), "<( a * c ) --> x>");
    assert_eq!((tv.f - 1.0).abs() < 0.01, true);
    assert_eq!((tv.c - 0.9).abs() < 0.01, true);
    assert_eq!(punct, EnumPunctation::JUGEMENT);
  }

  #[test]
  pub fn prod3() {
    let narsese = "<(a*c*z) --> x>.".to_string();
    let parseResOpt: Option<(Term, Tv, EnumPunctation, bool)> = parseNarsese(&narsese);
    assert_eq!(parseResOpt.is_some(), true);
    
    let (term, tv, punct, _isEvent) = parseResOpt.unwrap();
    assert_eq!(convTermToStr(&term), "<( a * c * z ) --> x>");
    assert_eq!((tv.f - 1.0).abs() < 0.01, true);
    assert_eq!((tv.c - 0.9).abs() < 0.01, true);
    assert_eq!(punct, EnumPunctation::JUGEMENT);
  }

  #[test]
  pub fn prod4() {
    let narsese = "<(a*c*z*y) --> x>.".to_string();
    let parseResOpt: Option<(Term, Tv, EnumPunctation, bool)> = parseNarsese(&narsese);
    assert_eq!(parseResOpt.is_some(), true);
    
    let (term, tv, punct, _isEvent) = parseResOpt.unwrap();
    assert_eq!(convTermToStr(&term), "<( a * c * z * y ) --> x>");
    assert_eq!((tv.f - 1.0).abs() < 0.01, true);
    assert_eq!((tv.c - 0.9).abs() < 0.01, true);
    assert_eq!(punct, EnumPunctation::JUGEMENT);
  }

  #[test]
  pub fn intint2() {
    let narsese = "<(a|c) --> x>.".to_string();
    let parseResOpt: Option<(Term, Tv, EnumPunctation, bool)> = parseNarsese(&narsese);
    assert_eq!(parseResOpt.is_some(), true);
    
    let (term, tv, punct, _isEvent) = parseResOpt.unwrap();
    assert_eq!(convTermToStr(&term), "<( a | c ) --> x>");
    assert_eq!((tv.f - 1.0).abs() < 0.01, true);
    assert_eq!((tv.c - 0.9).abs() < 0.01, true);
    assert_eq!(punct, EnumPunctation::JUGEMENT);
  }
  

  #[test]
  pub fn conj2_0() {
    let narsese = "<(a&&c) ==> x>.".to_string();
    let parseResOpt: Option<(Term, Tv, EnumPunctation, bool)> = parseNarsese(&narsese);
    assert_eq!(parseResOpt.is_some(), true);
    
    let (term, tv, punct, _isEvent) = parseResOpt.unwrap();
    assert_eq!(convTermToStr(&term), "<( a && c ) ==> x>");
    assert_eq!((tv.f - 1.0).abs() < 0.01, true);
    assert_eq!((tv.c - 0.9).abs() < 0.01, true);
    assert_eq!(punct, EnumPunctation::JUGEMENT);
  }

  #[test]
  pub fn conj2_1() {
    let narsese = "<(<a --> b>&&c) ==> x>.".to_string();
    let parseResOpt: Option<(Term, Tv, EnumPunctation, bool)> = parseNarsese(&narsese);
    assert_eq!(parseResOpt.is_some(), true);
    
    let (term, tv, punct, _isEvent) = parseResOpt.unwrap();
    assert_eq!(convTermToStr(&term), "<( <a --> b> && c ) ==> x>");
    assert_eq!((tv.f - 1.0).abs() < 0.01, true);
    assert_eq!((tv.c - 0.9).abs() < 0.01, true);
    assert_eq!(punct, EnumPunctation::JUGEMENT);
  }

  #[test]
  pub fn conj3_0() {
    let narsese = "<(a&&c&&d) ==> x>.".to_string();
    let parseResOpt: Option<(Term, Tv, EnumPunctation, bool)> = parseNarsese(&narsese);
    assert_eq!(parseResOpt.is_some(), true);
    
    let (term, tv, punct, _isEvent) = parseResOpt.unwrap();
    assert_eq!(convTermToStr(&term), "<( a && c && d ) ==> x>");
    assert_eq!((tv.f - 1.0).abs() < 0.01, true);
    assert_eq!((tv.c - 0.9).abs() < 0.01, true);
    assert_eq!(punct, EnumPunctation::JUGEMENT);
  }

  #[test]
  pub fn conj4_0() {
    let narsese = "<(a&&c&&d&&e) ==> x>.".to_string();
    let parseResOpt: Option<(Term, Tv, EnumPunctation, bool)> = parseNarsese(&narsese);
    assert_eq!(parseResOpt.is_some(), true);
    
    let (term, tv, punct, _isEvent) = parseResOpt.unwrap();
    assert_eq!(convTermToStr(&term), "<( a && c && d && e ) ==> x>");
    assert_eq!((tv.f - 1.0).abs() < 0.01, true);
    assert_eq!((tv.c - 0.9).abs() < 0.01, true);
    assert_eq!(punct, EnumPunctation::JUGEMENT);
  }
  
  #[test]
  pub fn seqPredImpl() {
    let narsese = "<(a,b) =/> c>.".to_string();
    let parseResOpt: Option<(Term, Tv, EnumPunctation, bool)> = parseNarsese(&narsese);
    assert_eq!(parseResOpt.is_some(), true);
    
    let (term, tv, punct, _isEvent) = parseResOpt.unwrap();
    assert_eq!(convTermToStr(&term), "<( a , b ) =/> c>");
    assert_eq!((tv.f - 1.0).abs() < 0.01, true);
    assert_eq!((tv.c - 0.9).abs() < 0.01, true);
    assert_eq!(punct, EnumPunctation::JUGEMENT);
  }


  // goal without temporal and TV
  #[test]
  pub fn goalWithoutTv() {
    let narsese = "a-c!".to_string();
    let parseResOpt: Option<(Term, Tv, EnumPunctation, bool)> = parseNarsese(&narsese);
    assert_eq!(parseResOpt.is_some(), true);
    
    let (term, _tv, punct, _isEvent) = parseResOpt.unwrap();
    assert_eq!(convTermToStr(&term), "a-c");
    assert_eq!(punct, EnumPunctation::GOAL);
  }


  #[test]
  pub fn neg_0() {
    let narsese = "(!a).".to_string();
    let parseResOpt: Option<(Term, Tv, EnumPunctation, bool)> = parseNarsese(&narsese);
    assert_eq!(parseResOpt.is_some(), true);
    
    let (term, tv, punct, _isEvent) = parseResOpt.unwrap();
    assert_eq!(convTermToStr(&term), "(! a )");
    assert_eq!((tv.f - 1.0).abs() < 0.01, true);
    assert_eq!((tv.c - 0.9).abs() < 0.01, true);
    assert_eq!(punct, EnumPunctation::JUGEMENT);
  }
}



fn ok1(input: &str) -> Result<&str, std::num::ParseIntError> {
  Ok(input)
}

fn isValidSign(c: char) -> bool {
  c.is_alphanumeric() || c == '-' || c == '_' || c == '^'
}

fn alpha2(input: &str) -> IResult<&str, &str> {
  map_res(
    take_while_m_n(1, 512, isValidSign),
    ok1
  )(input)
}

fn a(input:&str)  -> IResult<&str, Term> {
  let (input, _) = tag("{")(input)?;
  let (input, termContent) = parseSubjOrPred(input, true)?; // many_m_n!(1, 3, tag("a"))(input)?;
  let (input, _) = tag("}")(input)?;

  Ok((input, Term::SetExt(vec![Box::new(termContent)])))  // return {termContent}
}

fn b(input:&str)  -> IResult<&str, Term> {
  let (input, termContent) = alpha2(input)?;
  Ok((input, Term::Name(termContent.to_string())))
}


fn c(input:&str)  -> IResult<&str, Term> {
  let (input, _) = tag("[")(input)?;
  let (input, termContent) = parseSubjOrPred(input, true)?;
  let (input, _) = tag("]")(input)?;

  Ok((input, Term::SetInt(vec![Box::new(termContent)])))  // return {termContent}
}


fn qVar(input:&str) -> IResult<&str, Term> {
  let (input, _) = tag("?")(input)?;
  let (input, name) = alpha2(input)?;
  Ok((input, Term::QVar(name.to_string())))
}
fn depVar(input:&str) -> IResult<&str, Term> {
  let (input, _) = tag("#")(input)?;
  let (input, name) = alpha2(input)?;
  Ok((input, Term::DepVar(name.to_string())))
}
fn indepVar(input:&str) -> IResult<&str, Term> {
  let (input, _) = tag("$")(input)?;
  let (input, name) = alpha2(input)?;
  Ok((input, Term::IndepVar(name.to_string())))
}

// /param enStatement enable parsing of statement
fn parseSubjOrPred(input: &str, _enStatement:bool) -> IResult<&str, Term> {
  {
    let res0 = a(input);
    match res0 {
      Ok(term) => {
        return Ok(term.clone())
      },
      Err(_) => {}, // try other choice
    }
  }

  {
    let res0 = parseSeq2(input);
    match res0 {
      Ok(term) => {
        return Ok(term.clone())
      },
      Err(_) => {}, // try other choice
    }
  }

  {
    let res0 = parseIntInt2(input);
    match res0 {
      Ok(term) => {
        return Ok(term.clone())
      },
      Err(_) => {}, // try other choice
    }
  }

  {
    let res0 = c(input);
    match res0 {
      Ok(term) => {
        return Ok(term.clone())
      },
      Err(_) => {}, // try other choice
    }
  }

  {
    let res0 = parseNeg(input);
    match res0 {
      Ok(term) => {
        return Ok(term.clone())
      },
      Err(_) => {}, // try other choice
    }
  }

  {
    let res0 = parseProd(input);
    match res0 {
      Ok(term) => {
        return Ok(term.clone())
      },
      Err(_) => {}, // try other choice
    }
  }

  {
    let res0 = parseIntInt2(input);
    match res0 {
      Ok(term) => {
        return Ok(term.clone())
      },
      Err(_) => {}, // try other choice
    }
  }

  {
    let res0 = parseConj2(input);
    match res0 {
      Ok(term) => {
        return Ok(term.clone())
      },
      Err(_) => {}, // try other choice
    }
  }
  {
    let res0 = parseConj3(input);
    match res0 {
      Ok(term) => {
        return Ok(term.clone())
      },
      Err(_) => {}, // try other choice
    }
  }
  {
    let res0 = parseConj4(input);
    match res0 {
      Ok(term) => {
        return Ok(term.clone())
      },
      Err(_) => {}, // try other choice
    }
  }

  {
    let res0 = parseStatement(input);
    match res0 {
      Ok(term) => {
        return Ok(term.clone())
      },
      Err(_) => {}, // try other choice
    }
  }

  {
    let res0 = qVar(input);
    match res0 {
      Ok(term) => {
        return Ok(term.clone())
      },
      Err(_) => {}, // try other choice
    }
  }

  {
    let res0 = depVar(input);
    match res0 {
      Ok(term) => {
        return Ok(term.clone())
      },
      Err(_) => {}, // try other choice
    }
  }

  {
    let res0 = indepVar(input);
    match res0 {
      Ok(term) => {
        return Ok(term.clone())
      },
      Err(_) => {}, // try other choice
    }
  }

  return b(input)
}



fn copInh(input:&str)  -> IResult<&str, Copula> {
  let (input, _) = tag(" --> ")(input)?;
  Ok((input, Copula::INH))
}
fn copSim(input:&str)  -> IResult<&str, Copula> {
  let (input, _) = tag(" <-> ")(input)?;
  Ok((input, Copula::SIM))
}
fn copImpl(input:&str)  -> IResult<&str, Copula> {
  let (input, _) = tag(" ==> ")(input)?;
  Ok((input, Copula::IMPL))
}
fn copPredImpl(input:&str)  -> IResult<&str, Copula> {
  let (input, _) = tag(" =/> ")(input)?;
  Ok((input, Copula::PREDIMPL))
}
fn copEquiv(input:&str)  -> IResult<&str, Copula> {
  let (input, _) = tag(" <=> ")(input)?;
  Ok((input, Copula::EQUIV))
}

fn parseCopula(input: &str) -> IResult<&str, Copula> {
  {
    let res0 = copInh(input);
    match res0 {
      Ok(_X) => {
        return res0;
      },
      Err(_) => {}, // try other choice
    }
  }
  {
    let res0 = copSim(input);
    match res0 {
      Ok(_X) => {
        return res0;
      },
      Err(_) => {}, // try other choice
    }
  }
  {
    let res0 = copImpl(input);
    match res0 {
      Ok(_X) => {
        return res0;
      },
      Err(_) => {}, // try other choice
    }
  }
  {
    let res0 = copPredImpl(input);
    match res0 {
      Ok(_X) => {
        return res0;
      },
      Err(_) => {}, // try other choice
    }
  }
  
  return copEquiv(input);
}


pub fn parseNeg(input: &str) -> IResult<&str, Term> {
  let (input, _) = tag("(!")(input)?; // neg as in ONA
  let (input, a) = parseSubjOrPred(input, true)?;//parse0(input)?;
  let (input, _) = tag(")")(input)?;
  Ok((input, Term::Neg(Box::new(a.clone()))))
}

pub fn parseSeq2(input: &str) -> IResult<&str, Term> {
  let (input, _) = tag("(")(input)?;
  let (input, a) = parseSubjOrPred(input, true)?;//parse0(input)?;
  let (input, _) = tag(",")(input)?;
  let (input, b) = parseSubjOrPred(input, true)?;//parse0(input)?;
  let (input, _) = tag(")")(input)?;
  Ok((input, Term::Seq([&a,&b].iter().map(|v| Box::new((*v).clone())).collect())))
}

pub fn parseIntInt2(input: &str) -> IResult<&str, Term> {
  let (input, _) = tag("(")(input)?;
  let (input, a) = parseSubjOrPred(input, true)?;//parse0(input)?;
  let (input, _) = tag("|")(input)?;
  let (input, b) = parseSubjOrPred(input, true)?;//parse0(input)?;
  let (input, _) = tag(")")(input)?;
  Ok((input, Term::IntInt([&a,&b].iter().map(|v| Box::new((*v).clone())).collect())))
}


// parses product with two components
pub fn parseProd(input: &str) -> IResult<&str, Term> {
  let (input, _) = tag("(")(input)?;
  let mut subterms = vec![];
  let (input, a) = parseSubjOrPred(input, true)?;
  subterms.push(a.clone());
  let (input, _) = tag("*")(input)?;
  
  /*
  let mut type_:Option<&str> = None;

  {
    let res0 = conStar(input);
    match res0 {
      Ok((_,X)) => {
        type_ = Some(X);
      },
      Err(_) => {}, // try other choice
    }
  }

  *
  if !type_.is_some() {
    let res0 = conConj(input);
    match res0 {
      Ok((_,X)) => {
        type_ = Some(X);
      },
      Err(_) => {}, // try other choice
    }
  }
  *

  if !type_.is_some() {
    return Err(nom::Err::Incomplete(nom::Needed::Unknown)); // propagate error
  }
  
  */


  let (mut input, b) = parseSubjOrPred(input, true)?;
  subterms.push(b.clone());

  loop { // loop for more sub-terms
    let res0: IResult<&str, &str> = tag("*")(input);
    match res0 {
      Ok((input2, _)) => {
        input = input2;
      },
      Err(_) => {
        break;
      },
    };

    let (input2, subterm) = parseSubjOrPred(input, true)?;
    input = input2;
    subterms.push(subterm.clone());
  }

  let (input2, _) = tag(")")(input)?;
  input = input2;
  Ok((input, p(&subterms)))
}


// parses product with two components
pub fn parseConj2(input: &str) -> IResult<&str, Term> {
  let (input, _) = tag("(")(input)?;
  let (input, a) = parseSubjOrPred(input, true)?;
  let (input, _) = tag("&&")(input)?;
  let (input, b) = parseSubjOrPred(input, true)?;
  let (input, _) = tag(")")(input)?;
  Ok((input, conj(&vec![a, b])))
}
pub fn parseConj3(input: &str) -> IResult<&str, Term> {
  let (input, _) = tag("(")(input)?;
  let (input, a) = parseSubjOrPred(input, true)?;
  let (input, _) = tag("&&")(input)?;
  let (input, b) = parseSubjOrPred(input, true)?;
  let (input, _) = tag("&&")(input)?;
  let (input, c) = parseSubjOrPred(input, true)?;
  let (input, _) = tag(")")(input)?;
  Ok((input, conj(&vec![a, b, c])))
}
pub fn parseConj4(input: &str) -> IResult<&str, Term> {
  let (input, _) = tag("(")(input)?;
  let (input, a) = parseSubjOrPred(input, true)?;
  let (input, _) = tag("&&")(input)?;
  let (input, b) = parseSubjOrPred(input, true)?;
  let (input, _) = tag("&&")(input)?;
  let (input, c) = parseSubjOrPred(input, true)?;
  let (input, _) = tag("&&")(input)?;
  let (input, d) = parseSubjOrPred(input, true)?;
  let (input, _) = tag(")")(input)?;
  Ok((input, conj(&vec![a, b, c, d])))
}

pub fn parseStatement(input: &str) -> IResult<&str, Term> {
  let (input, _) = tag("<")(input)?;
  //let (input, subj) = tag("b")(input)?;
  
  //let (input, _) = tag("{")(input)?;
  //let (input, subj) = alpha2(input)?; // many_m_n!(1, 3, tag("a"))(input)?;
  //let (input, _) = tag("}")(input)?;
  let (input, subj) = parseSubjOrPred(input, true)?;

  //let (input, _) = tag(" --> ")(input)?; // TODO< remove spaces >
  let (input, copula) = parseCopula(input)?;

  //let (input, pred) = alpha2(input)?;
  let (input, pred) = parseSubjOrPred(input, true)?;
  
  let (input, _) = tag(">")(input)?;

  Ok((input, s(copula, &subj, &pred)))
}

pub fn parseEntry(input: &str) -> IResult<&str, Term> {
  parseSubjOrPred(input, true) // can be statement or single term etc.
}
