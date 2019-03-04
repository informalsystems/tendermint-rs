use std::{
    path::Path,
    fs::File,
    io::prelude::*,
};
use tendermint::chain;
use serde_json;
use abscissa::Error;


#[derive(Serialize,Deserialize)]
struct LastSignData{
    pub height: i64,
    pub round: i64,
    pub step : i8,
    pub signature: String,
    pub signbytes: String,      
}

pub struct LastSignState{
    data: LastSignData,
    file: File,
    chain_id: chain::Id
}

/// Error type
#[derive(Debug)]
pub struct LastSignError(Error<LastSignErrorKind>);

#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum LastSignErrorKind{
    #[fail(display = "height regression")]
    HeightRegression,
    #[fail(display = "step regression")]
    StepRegression,
    #[fail(display = "round regression")]
    RoundRegression,
}

impl From<Error<LastSignErrorKind>> for LastSignError {
    fn from(other: Error<LastSignErrorKind>) -> Self {
        LastSignError(other)
    }
}

impl LastSignState{
    pub fn init_state(&mut self, path: &Path) -> std::io::Result<()>{
        if path.exists(){
            return Ok(());
        }
        self.file = File::create(self.filename())?;
        self.file.write_all(serde_json::to_string(&self.data).unwrap().as_ref())?;
        self.file.sync_all()?;
        return Ok(());
    }

   pub fn load_state(&mut self, path:&Path) -> std::io::Result<()>{
        self.file = File::open(path)?;
        let mut contents = String::new();
        self.file.read_to_string(&mut contents)?;
        self.data = serde_json::from_str(&contents).unwrap();       
        return Ok(());    
    }
    
   pub fn filename(&self) -> String{
        self.chain_id.as_str().to_owned() + "_validator_state.json"
    }

   pub fn check_and_update_hrs(&mut self, height: i64, round:i64, step:i8)->Result<(),LastSignError>{
       if height < self.data.height{
            fail!(HeightRegression, "last height:{} new height:{}", self.data.height, height);
       } 
       if height == self.data.height {
            if  round < self.data.round{
                fail!(RoundRegression, "round regression at height:{} last round:{} new round:{}",height, self.data.round, round)
            }
            if round == self.data.round{
                if step < self.data.step{
                        fail!(StepRegression, "round regression at height:{} round:{} last step:{} new step:{}",height,round, self.data.step, step)
                }
            }
       }
        self.data.height = height;
        self.data.round = round;
        self.data.step = step;
        Ok(())
   }


}