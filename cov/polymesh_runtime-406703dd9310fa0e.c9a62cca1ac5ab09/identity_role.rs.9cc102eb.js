var data = {lines:[
{"lineNum":"    1","line":"use parity_scale_codec::{Decode, Encode};"},
{"lineNum":"    2","line":""},
{"lineNum":"    3","line":"/// Identity roles."},
{"lineNum":"    4","line":"/// # TODO"},
{"lineNum":"    5","line":"/// 2. Review documents:"},
{"lineNum":"    6","line":"///     - [MESH-235](https://polymath.atlassian.net/browse/MESH-235)"},
{"lineNum":"    7","line":"///     - [Polymesh: Roles/Permissions](https://docs.google.com/document/d/12u-rMavow4fvidsFlLcLe7DAXuqWk8XUHOBV9kw05Z8/)"},
{"lineNum":"    8","line":"#[allow(missing_docs)]"},
{"lineNum":"    9","line":"#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]","class":"lineNoCov","hits":"0","possible_hits":"29",},
{"lineNum":"   10","line":"pub enum IdentityRole {","class":"lineNoCov","hits":"0","possible_hits":"12",},
{"lineNum":"   11","line":"    Issuer,","class":"lineNoCov","hits":"0","possible_hits":"2",},
{"lineNum":"   12","line":"    SimpleTokenIssuer,","class":"lineNoCov","hits":"0","possible_hits":"2",},
{"lineNum":"   13","line":"    Validator,","class":"lineNoCov","hits":"0","possible_hits":"2",},
{"lineNum":"   14","line":"    ClaimIssuer,","class":"lineNoCov","hits":"0","possible_hits":"2",},
{"lineNum":"   15","line":"    // From MESH-235"},
{"lineNum":"   16","line":"    Investor,","class":"lineNoCov","hits":"0","possible_hits":"2",},
{"lineNum":"   17","line":"    NodeRunner,","class":"lineNoCov","hits":"0","possible_hits":"2",},
{"lineNum":"   18","line":"    PM,","class":"lineNoCov","hits":"0","possible_hits":"2",},
{"lineNum":"   19","line":"    KYCAMLClaimIssuer,","class":"lineNoCov","hits":"0","possible_hits":"2",},
{"lineNum":"   20","line":"    AccreditedInvestorClaimIssuer,","class":"lineNoCov","hits":"0","possible_hits":"2",},
{"lineNum":"   21","line":"    VerifiedIdentityClaimIssuer,","class":"lineNoCov","hits":"0","possible_hits":"2",},
{"lineNum":"   22","line":"    Custom(u8),","class":"lineNoCov","hits":"0","possible_hits":"6",},
{"lineNum":"   23","line":"}"},
]};
var percent_low = 25;var percent_high = 75;
var header = { "command" : "polymesh_runtime-406703dd9310fa0e", "date" : "2019-11-06 13:35:33", "instrumented" : 13, "covered" : 0,};
var merged_data = [];
