#[derive(Debug)]
pub enum ExecutionPlan {
    Execute(String, Vec<String>),
    Pipe(Box<ExecutionPlan>, Box<ExecutionPlan>),
    And(Box<ExecutionPlan>, Box<ExecutionPlan>),
    Or(Box<ExecutionPlan>, Box<ExecutionPlan>),
    Background(Box<ExecutionPlan>),
    RedirectPipe(Box<ExecutionPlan>, PipeRedirection),
    NoOp,
}

#[derive(Debug)]
pub struct PipeRedirection {
    pub from: PipeType,
    pub to: PipeType,
    pub append: bool,
}

#[derive(Debug)]
pub enum PipeType {
    Stdout,
    Stderr,
    Stdin,
    Null,
    File(String),
}
