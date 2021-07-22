let Person =
      { Type = { name : Text, tags : List Text, gitLink : Text, twitter : Text }
      , default =
        { name = "", tags = [] : List Text, gitLink = "", twitter = "" }
      }

let Author =
      { Type =
          { name : Text
          , handle : Text
          , picUrl : Optional Text
          , link : Optional Text
          , twitter : Optional Text
          , default : Bool
          , inSystem : Bool
          }
      , default =
        { name = ""
        , handle = ""
        , picUrl = None Text
        , link = None Text
        , twitter = None Text
        , default = False
        , inSystem = True
        }
      }

let defaultPort = env:PORT ? 3030

let defaultWebMentionEndpoint =
        env:WEBMENTION_ENDPOINT
      ? "https://mi.within.website/api/webmention/accept"

let Config =
      { Type =
          { signalboost : List Person.Type
          , authors : List Author.Type
          , port : Natural
          , clackSet : List Text
          , resumeFname : Text
          , webMentionEndpoint : Text
          , miToken : Text
          }
      , default =
        { signalboost = [] : List Person.Type
        , authors =
          [ Author::{
            , name = "fetsorn"
            , handle = "fetsorn"
            , picUrl = Some "/static/img/avatar.png"
            , link = Some "https://fetsorn.website"
            , twitter = Some "fetsorn"
            , default = True
            , inSystem = True
            }
          ]
        , port = defaultPort
        , clackSet = [ "Ashlynn" ]
        , resumeFname = "./static/resume/resume.md"
        , webMentionEndpoint = defaultWebMentionEndpoint
        , miToken = "${env:MI_TOKEN as Text ? ""}"
        }
      }

in  Config::{
    , signalboost = ./signalboost.dhall
    , clackSet =
      [ "Ashlynn", "Terry Davis", "Dennis Ritchie", "Steven Hawking" ]
    }
