// patham9 comptible pong

use rand::Rng;
use rand::rngs::ThreadRng;

pub struct EnvState {
    pub szX:i64,
    pub szY:i64,
    pub ballX:i64,
    pub ballY:i64,
    pub batX:i64,
    pub batVX:i64,
    pub batWidth:i64, //"radius", batWidth from middle to the left and right
    pub virtualBatWidth:i64, // bat width used for center computation
    pub vX:i64,
    pub vY:i64,
    pub mulVX:i64, // multiply x velocity by this before moving
    pub hits:i64,
    pub misses:i64,
    pub t:i64,
    
    pub verbosity:i64,
}

pub fn makeEnvState() -> EnvState {
    let szX = 50;
    let szY = 20;

    EnvState {
        szX:szX,
        szY:szY,
        ballX:szX/2,
        ballY:szY/5,
        batX:20,
        batVX:0,
        batWidth:6,
        virtualBatWidth:6,
        vX:1,
        vY:1,
        mulVX:0,
        hits:0,
        misses:0,
        t:0,
        verbosity:1,
    }
}


pub fn simStep(env:&mut EnvState, rng:&mut ThreadRng) {
    env.t+=1;

    /* commented because we don't visualize!
    //fputs("\033[1;1H\033[2J", stdout); //POSIX clear screen
    for(int i=0; i<batX-batWidth+1; i++)
    {
        fputs(" ", stdout);
    }
    for(int i=0; i<batWidth*2-1+MIN(0,batX) ;i++)
    {
        fputs("@", stdout);
    }
    puts("");
    for(int i=0; i<ballY; i++)
    {
        for(int k=0; k<szX; k++)
        {
            fputs(" ", stdout);
        }
        puts("|");
    }
    for(int i=0; i<ballX; i++)
    {
        fputs(" ", stdout);
    }
    fputs("#", stdout);
    for(int i=ballX+1; i<szX; i++)
    {
        fputs(" ", stdout);
    }
    puts("|");
    for(int i=ballY+1; i<szY; i++)
    {
        for(int k=0; k<szX; k++)
        {
            fputs(" ", stdout);
        }
        puts("|");
    }
    */


    /*
    if(batX <= ballX - virtualBatWidth) {
        if(true) TerminalOut.out('BALL right');
        reasoner.input("<{right}-->ball>. :|:");
    }
    else if(ballX + virtualBatWidth < batX) {
        if(true) TerminalOut.out('BALL right');
        reasoner.input("<{left}-->ball>. :|:");
    }
    else {
        if(true) TerminalOut.out('BALL center');
        reasoner.input("<{center}-->ball>. :|:");
    }
    reasoner.input("<good-->nar>! :|:");
    */

    if env.ballX <= 0 {
        env.vX = 1;
    }
    if env.ballX >= env.szX-1 {
        env.vX = -1;
    }
    if env.ballY <= 0 {
        env.vY = 1;
    }
    if env.ballY >= env.szY-1 {
        env.vY = -1;
    }
    if (env.t%2) == 1 {
        env.ballX += env.vX;
    }
    env.ballX += env.vX*env.mulVX;
    env.ballY += env.vY;
    if env.ballY == 0 {
        if (env.ballX-env.batX).abs() <= env.batWidth {
            //reasoner.input("<good-->nar>. :|:");
            if env.verbosity > 0 {println!("env: good")};
            env.hits+=1;
        }
        else {
            if env.verbosity > 0 {println!("env: bad")};
            env.misses+=1;
        }
    }
    if env.ballY == 0 || env.ballX == 0 || env.ballX >= env.szX-1 {
        env.ballY = (env.szY/2)+rng.gen_range(0, env.szY/2);
        env.ballX = rng.gen_range(0,env.szX);
        env.vX = if rng.gen_range(0,2) == 0 {1} else {-1};

        println!("env: respawn ball");
    }
    /*
    if(opLeft.triggered) {
        opLeft.triggered = false;
        TerminalOut.out("Exec: op_left");
        batVX = -3;
    }
    if(opRight.triggered) {
        opRight.triggered = false;
        TerminalOut.out("Exec: op_right");
        batVX = 3;
    }
    */

    /*
    if(NAR_Pong_Stop_executed) {
        NAR_Pong_Stop_executed = false;
        TerminalOut.out("Exec: op_stop");
        batVX = 0;
    }*/

    //env.batX+=env.batVX; // move bat

    let h0 = (env.szX-1+env.batWidth).min(env.batX+env.batVX*env.batWidth/2);
    env.batX=/*Std.int*/(env.batWidth*2).max(h0);
    let mut ratio:f64 = env.hits as f64;
    ratio /= (env.hits + env.misses) as f64;
    if env.verbosity > 0 {println!("PONG  Hits={} misses={} ratio={} time={}", env.hits, env.misses, ratio, env.t)};
}
