let Person =
      { Type = { name : Text, tags : List Text, gitLink : Text, twitter : Text }
      , default =
        { name = "", tags = [] : List Text, gitLink = "", twitter = "" }
      }

in  [ Person::{
      , name = "Name Surname"
      , tags =
        [ "tech1"
        , "tech2"
        ]
      , gitLink = "https://github.com/username"
      , twitter = "https://twitter.com/username"
      }
    ]
