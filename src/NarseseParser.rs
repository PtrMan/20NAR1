
use nom::{
  IResult,
  bytes::complete::{tag, take_while_m_n},
  combinator::map_res,
//  sequence::tuple,
//  take_while,
//  alt,
};
//use nom::character::is_alphanumeric;
//use nom::named;
//use nom::many_m_n;

use Term::*;
use NarSentence::EnumPunctation;
use Tv::Tv;
use TermApi::*;

// finds out if narsese has tv and returns TV if TV exists
// cuts away narsese of TV if TV is detected
pub fn parseNarseseRetTv(narsese:&mut String, tv:&mut Tv) {
  *tv = Tv{f:1.0,c:0.9}; // set TV to default
  
  if narsese.chars().nth(narsese.len()-1).unwrap() == '}' {
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

pub fn parseNarsese(narsese:&String) -> Option<(Term, Tv, EnumPunctation)> {
  let mut narsese2:String = narsese.clone();

  let mut tv = Tv{f:1.0,c:0.9};
  parseNarseseRetTv(&mut narsese2, &mut tv);
  println!("{}", &narsese2);
  narsese2 = narsese2.trim_end().to_string();
  println!("{}", &narsese2);
  
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

  let parsed:IResult<&str, Term> = parse0(&narseseInner);

  match parsed {
    Ok((str2, term)) => {
      Some((term, tv, punctation))
    },
    Err(_) => {
      None
    }
  }
}





#[cfg(test)]
mod tests {
  use super::*;
  use Term::convTermToStr;

  #[test]
  pub fn withTv() {
    let narsese = "<a --> b>. {0.4 0.8}".to_string();
    let parseResOpt: Option<(Term, Tv, EnumPunctation)> = parseNarsese(&narsese);
    assert_eq!(parseResOpt.is_some(), true);
    
    let (term, tv, punct) = parseResOpt.unwrap();
    assert_eq!(convTermToStr(&term), "<a --> b>");
    assert_eq!((tv.f - 0.4).abs() < 0.01, true);
    assert_eq!((tv.c - 0.8).abs() < 0.01, true);
    assert_eq!(punct, EnumPunctation::JUGEMENT);
  }

  #[test]
  pub fn withoutTv() {
    let narsese = "<a --> b>.".to_string();
    let parseResOpt: Option<(Term, Tv, EnumPunctation)> = parseNarsese(&narsese);
    assert_eq!(parseResOpt.is_some(), true);
    
    let (term, tv, punct) = parseResOpt.unwrap();
    assert_eq!(convTermToStr(&term), "<a --> b>");
    assert_eq!((tv.f - 1.0).abs() < 0.01, true);
    assert_eq!((tv.c - 0.9).abs() < 0.01, true);
    assert_eq!(punct, EnumPunctation::JUGEMENT);
  }


  #[test]
  pub fn setInt() {
    let narsese = "<a --> [b]>.".to_string();
    let parseResOpt: Option<(Term, Tv, EnumPunctation)> = parseNarsese(&narsese);
    assert_eq!(parseResOpt.is_some(), true);
    
    let (term, tv, punct) = parseResOpt.unwrap();
    assert_eq!(convTermToStr(&term), "<a --> [b]>");
    assert_eq!((tv.f - 1.0).abs() < 0.01, true);
    assert_eq!((tv.c - 0.9).abs() < 0.01, true);
    assert_eq!(punct, EnumPunctation::JUGEMENT);
  }

  #[test]
  pub fn setExt() {
    let narsese = "<{a} --> b>.".to_string();
    let parseResOpt: Option<(Term, Tv, EnumPunctation)> = parseNarsese(&narsese);
    assert_eq!(parseResOpt.is_some(), true);
    
    let (term, tv, punct) = parseResOpt.unwrap();
    assert_eq!(convTermToStr(&term), "<{a} --> b>");
    assert_eq!((tv.f - 1.0).abs() < 0.01, true);
    assert_eq!((tv.c - 0.9).abs() < 0.01, true);
    assert_eq!(punct, EnumPunctation::JUGEMENT);
  }
}



fn ok1(input: &str) -> Result<&str, std::num::ParseIntError> {
  Ok(input)
}

fn is_alphanumeric2(c: char) -> bool {
  c.is_alphanumeric()
}

fn alpha2(input: &str) -> IResult<&str, &str> {
  map_res(
    take_while_m_n(1, 512, is_alphanumeric2),
    ok1
  )(input)
}

fn a(input:&str)  -> IResult<&str, Term> {
  let (input, _) = tag("{")(input)?;
  let (input, termContent) = alpha2(input)?; // many_m_n!(1, 3, tag("a"))(input)?;
  let (input, _) = tag("}")(input)?;

  Ok((input, Term::SetExt(vec![Box::new(Term::Name(termContent.to_string()))])))  // return {termContent}
}

fn b(input:&str)  -> IResult<&str, Term> {
  let (input, termContent) = alpha2(input)?;
  Ok((input, Term::Name(termContent.to_string())))
}


fn c(input:&str)  -> IResult<&str, Term> {
  let (input, _) = tag("[")(input)?;
  let (input, termContent) = alpha2(input)?;
  let (input, _) = tag("]")(input)?;

  Ok((input, Term::SetInt(vec![Box::new(Term::Name(termContent.to_string()))])))  // return {termContent}
}


fn parseSubjOrPred(input: &str) -> IResult<&str, Term> {
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
    let res0 = c(input);
    match res0 {
      Ok(term) => {
        return Ok(term.clone())
      },
      Err(_) => {}, // try other choice
    }
  }

  {
    let res0 = parseProd2(input);
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
fn copEquiv(input:&str)  -> IResult<&str, Copula> {
  let (input, _) = tag(" <=> ")(input)?;
  Ok((input, Copula::EQUIV))
}

fn parseCopula(input: &str) -> IResult<&str, Copula> {
  {
    let res0 = copInh(input);
    match res0 {
      Ok(X) => {
        return res0;
      },
      Err(_) => {}, // try other choice
    }
  }
  {
    let res0 = copSim(input);
    match res0 {
      Ok(X) => {
        return res0;
      },
      Err(_) => {}, // try other choice
    }
  }
  {
    let res0 = copImpl(input);
    match res0 {
      Ok(X) => {
        return res0;
      },
      Err(_) => {}, // try other choice
    }
  }
  
  return copEquiv(input);
}


// parses product with two components
pub fn parseProd2(input: &str) -> IResult<&str, Term> {
  let (input, _) = tag("(")(input)?;
  let (input, a) = parseSubjOrPred(input)?;//parse0(input)?;
  let (input, _) = tag("*")(input)?;
  let (input, b) = parseSubjOrPred(input)?;//parse0(input)?;
  let (input, _) = tag(")")(input)?;
  Ok((input, p2(&a, &b)))
}

pub fn parse0(input: &str) -> IResult<&str, Term> {
  //parseProd2(input)

  
  //named!( alpha, take_while!( is_alphanumeric ) );

  let (input, _) = tag("<")(input)?;
  //let (input, subj) = tag("b")(input)?;
  
  //let (input, _) = tag("{")(input)?;
  //let (input, subj) = alpha2(input)?; // many_m_n!(1, 3, tag("a"))(input)?;
  //let (input, _) = tag("}")(input)?;
  let (input, subj) = parseSubjOrPred(input)?;

  //let (input, _) = tag(" --> ")(input)?; // TODO< remove spaces >
  let (input, copula) = parseCopula(input)?;

  //let (input, pred) = alpha2(input)?;
  let (input, pred) = parseSubjOrPred(input)?;
  
  let (input, _) = tag(">")(input)?;

  Ok((input, s(copula, &subj, &pred)))
}

