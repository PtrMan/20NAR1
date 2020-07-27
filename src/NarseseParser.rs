
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



// sseehh API
pub fn p2(a:&Term,b:&Term)->Term {
  Term::Prod(vec![Box::new(a.clone()), Box::new(b.clone())])
}


// finds out if narsese has tv and returns TV if TV exists
// cuts away narsese of TV if TV is detected
// TODO REFACTOR< return option of TV >
pub fn parseNarseseRetTv(narsese:&mut String, f:&mut f64,c:&mut f64,hasTv:&mut bool) {
    *hasTv = false;
    
    if narsese.chars().nth(narsese.len()-1).unwrap() == '}' {
        for revIdx in 1..narsese.len() { // scan for '{'
            let idx = narsese.len() - revIdx; // compute index from back
            
            if narsese.chars().nth(idx).unwrap() == '{' {
                let tvStr = &narsese[idx+1..narsese.len()-1];
                let splitted:Vec<&str> = tvStr.split(" ").collect();
                
                if splitted.len() == 2 { // must have two values
                    // TODO< handle error better >
                    *f = splitted[0].parse::<f64>().unwrap();
                    *c = splitted[1].parse::<f64>().unwrap();
                    *hasTv = true;
                    *narsese = narsese[..idx].to_string(); // cut away
                    return;
                }
                return;
            }
        }
    }
}


// test for parsing of TV
pub fn mainX() {
    let mut f = 1.0;
    let mut c = 0.9;
    let mut hasTv = false;
    let mut narsese = "<a --> {b}>. {1.0 0.9}".to_string();
    parseNarseseRetTv(&mut narsese, &mut f, &mut c, &mut hasTv);
    println!("{}", &narsese);
    narsese = narsese.trim_right().to_string();
    println!("{}", &narsese);
    
    let punctation = narsese.chars().nth(narsese.len()-1).unwrap();
    let narseseInner = narsese[..narsese.len()-1].to_string();
    
    println!("{}   {}", narseseInner, punctation);
    println!("f = {}", f);
    println!("c = {}", c);

    let parsed:IResult<&str, Term> = parse0(&narseseInner);
    
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

  Ok((input, Term::Cop(copula, Box::new(subj), Box::new(pred))))
}

