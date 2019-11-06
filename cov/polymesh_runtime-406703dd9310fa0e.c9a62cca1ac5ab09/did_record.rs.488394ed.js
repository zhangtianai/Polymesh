var data = {lines:[
{"lineNum":"    1","line":"use parity_scale_codec::{Decode, Encode};"},
{"lineNum":"    2","line":"use rstd::prelude::Vec;"},
{"lineNum":"    3","line":""},
{"lineNum":"    4","line":"use crate::{IdentityRole, Key, SigningKey};"},
{"lineNum":"    5","line":""},
{"lineNum":"    6","line":"/// Identity information."},
{"lineNum":"    7","line":"#[allow(missing_docs)]"},
{"lineNum":"    8","line":"#[derive(Encode, Decode, Default, Clone, PartialEq, Debug)]","class":"linePartCov","hits":"10","order":"5379","possible_hits":"17",},
{"lineNum":"    9","line":"pub struct DidRecord<U> {","class":"lineCov","hits":"1","order":"5521","possible_hits":"1",},
{"lineNum":"   10","line":"    pub roles: Vec<IdentityRole>,","class":"linePartCov","hits":"3","order":"5380","possible_hits":"5",},
{"lineNum":"   11","line":"    pub master_key: Key,","class":"linePartCov","hits":"3","order":"5381","possible_hits":"6",},
{"lineNum":"   12","line":"    pub signing_keys: Vec<SigningKey>,","class":"linePartCov","hits":"2","order":"5382","possible_hits":"5",},
{"lineNum":"   13","line":"    pub balance: U,","class":"linePartCov","hits":"3","order":"5383","possible_hits":"7",},
{"lineNum":"   14","line":"}","class":"lineNoCov","hits":"0","possible_hits":"5",},
{"lineNum":"   15","line":""},
{"lineNum":"   16","line":"impl<U> DidRecord<U> {"},
{"lineNum":"   17","line":"    /// It checks if this entity contains role `role`."},
{"lineNum":"   18","line":"    pub fn has_role(&self, role: IdentityRole) -> bool {"},
{"lineNum":"   19","line":"        self.roles.contains(&role)"},
{"lineNum":"   20","line":"    }"},
{"lineNum":"   21","line":"}"},
]};
var percent_low = 25;var percent_high = 75;
var header = { "command" : "polymesh_runtime-406703dd9310fa0e", "date" : "2019-11-06 13:35:33", "instrumented" : 7, "covered" : 6,};
var merged_data = [];
