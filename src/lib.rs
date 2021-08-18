#![allow(non_snake_case)]
#![allow(dead_code)]

// NARS
pub mod BinSearch;
pub mod AeraishPerceptionComp;
pub mod Misc;
pub mod NarseseParser;
pub mod NarSentence;
pub mod NarMem;
pub mod NarWorkingCycle;
pub mod NarStamp;
pub mod Tv;
pub mod Term;
pub mod TermApi;
pub mod TermUtils;
pub mod Nar;
pub mod NarUnify;
pub mod NarProc;
pub mod NarGoalSystem;
pub mod NarInfProcedural;

// quality of life
pub mod NarInteractive;
//pub mod NarModuleNlp;
//pub mod NarModuleNlp2;
pub mod NarModuleNlp3;
pub mod NarServer;
pub mod NarInputFacade;
pub mod NarUtilReadn;

// env
pub mod EnvPong3;
pub mod ProcTicTacToe;
pub mod Reasoner1Entry;
pub mod ProcChaosEntry;

pub mod OpLib;

// eval
pub mod Eval;

// utils
pub mod Utils;
pub mod Map2d;

// ML
pub mod Nn;
//pub mod NnTrain;
//pub mod Classifier;
//pub mod expRepresent0;
pub mod ad;
pub mod mlutils;

//pub mod narPerception; // not include because file has issues

// NAL extensions
pub mod TvVec;

// Modules (batteries included)
pub mod ModNlpA;

pub mod ModVisionB;
pub mod ModVisionC;
pub mod ModVisionD;
