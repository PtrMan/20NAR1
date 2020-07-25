
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

    let parsed:IResult<&str, X> = parse0(&narseseInner);
    
}



#[derive(Debug,PartialEq)]
pub struct X {
  pub subj: String,
  pub pred: String,
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

fn a(input:&str)  -> IResult<&str, &str> {
  let (input, _) = tag("{")(input)?;
  let (input, subj) = alpha2(input)?; // many_m_n!(1, 3, tag("a"))(input)?;
  let (input, _) = tag("}")(input)?;

  Ok((input, subj))
}

fn b(input:&str)  -> IResult<&str, &str> {
  let (input, subj) = alpha2(input)?;
  Ok((input, subj))
}

fn parseSubjOrPred(input: &str) -> IResult<&str, &str> {
  let res0 = a(input);
  match res0 {
    Ok(X) => {
      return res0;
    },
    Err(_) => {}, // try other choice
  }
  
  return b(input)
  //alt!(a(input) | a(input))
  


  //alt!(a(input) | b(input))

  //named!( dragon_or_beast, alt!(tag("<")(input)) );
  
  //dragon_or_beast(input)
}



fn copInh(input:&str)  -> IResult<&str, &str> {
  let (input, _) = tag(" --> ")(input)?;
  Ok((input, ""))
}
fn copSim(input:&str)  -> IResult<&str, &str> {
  let (input, _) = tag(" <-> ")(input)?;
  Ok((input, ""))
}
fn copImpl(input:&str)  -> IResult<&str, &str> {
  let (input, _) = tag(" ==> ")(input)?;
  Ok((input, ""))
}
fn copEquiv(input:&str)  -> IResult<&str, &str> {
  let (input, _) = tag(" <=> ")(input)?;
  Ok((input, ""))
}

fn parseCopula(input: &str) -> IResult<&str, &str> {
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
  {
    return copEquiv(input);
  }
}



// TODO< return real term >
fn parse0(input: &str) -> IResult<&str, X> {
    //named!( alpha, take_while!( is_alphanumeric ) );

    let (input, _) = tag("<")(input)?;
    //let (input, subj) = tag("b")(input)?;
    
    //let (input, _) = tag("{")(input)?;
    //let (input, subj) = alpha2(input)?; // many_m_n!(1, 3, tag("a"))(input)?;
    //let (input, _) = tag("}")(input)?;
    let (input, subj) = parseSubjOrPred(input)?;

    //let (input, _) = tag(" --> ")(input)?; // TODO< remove spaces >
    let (input, _) = parseCopula(input)?;

    //let (input, pred) = alpha2(input)?;
    let (input, pred) = parseSubjOrPred(input)?;
    
    let (input, _) = tag(">")(input)?;
  
    Ok((input, X { subj:subj.to_string(), pred:pred.to_string()}))
}

