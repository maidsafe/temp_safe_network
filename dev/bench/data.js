window.BENCHMARK_DATA = {
  "lastUpdate": 1661175176645,
  "repoUrl": "https://github.com/maidsafe/safe_network",
  "entries": {
    "Safe Network Benchmarks": [
      {
        "commit": {
          "author": {
            "email": "joshuef@gmail.com",
            "name": "Josh Wilson",
            "username": "joshuef"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "697442816bcf96193a52955c97997a1972237bff",
          "message": "ci: another git hook removal attempt",
          "timestamp": "2022-04-15T13:02:44+02:00",
          "tree_id": "2a44948a1c42a2095b0433fa06357e3b43b1db01",
          "url": "https://github.com/maidsafe/safe_network/commit/697442816bcf96193a52955c97997a1972237bff"
        },
        "date": 1650022308342,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload 3072b",
            "value": 30701240334,
            "range": "± 12996145696",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 4699648042,
            "range": "± 15612645960",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 23625858546,
            "range": "± 16965174814",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joshuef@gmail.com",
            "name": "Josh Wilson",
            "username": "joshuef"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "efccd8f58e69d404d5445c0c8b86bac0612a6879",
          "message": "ci(bench): fix upload only clippy",
          "timestamp": "2022-04-18T08:25:49+02:00",
          "tree_id": "e79aa664b4daec75d9ea3036ead3ec0c20b344a8",
          "url": "https://github.com/maidsafe/safe_network/commit/efccd8f58e69d404d5445c0c8b86bac0612a6879"
        },
        "date": 1650265764877,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 19948440514,
            "range": "± 12157762917",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 6083047591,
            "range": "± 3447026656",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 8790251295,
            "range": "± 2000209165",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 8950750317,
            "range": "± 1188503277",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3133877755,
            "range": "± 601268216",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4575696139,
            "range": "± 237044270",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joshuef@gmail.com",
            "name": "Josh Wilson",
            "username": "joshuef"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "e9550c10e7cf5ff5778e32d8eeeddea09824ecec",
          "message": "fix: network_split example node start interval increased\n\nthis helps solidify network startup.",
          "timestamp": "2022-04-19T18:06:08+02:00",
          "tree_id": "c9e8e8317c1f3f3deb0ba99650deaecdbbe5478f",
          "url": "https://github.com/maidsafe/safe_network/commit/e9550c10e7cf5ff5778e32d8eeeddea09824ecec"
        },
        "date": 1650386805741,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10595772190,
            "range": "± 11492923238",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3854360116,
            "range": "± 156066345",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 9454732251,
            "range": "± 232399915",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10040193644,
            "range": "± 2219062592",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 4095784163,
            "range": "± 2088158105",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4710870422,
            "range": "± 240679584",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joshuef@gmail.com",
            "name": "Josh Wilson",
            "username": "joshuef"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "d632a2d4831ffcbc8394fc7bb26e9b40008e1a8e",
          "message": "ci: remove useless test --no-run step\n\nnow we use nextest this actually has no effect and is a waste of time,\nalso inceases the node unittest time as builds can take almost 10mins",
          "timestamp": "2022-04-20T08:47:24+02:00",
          "tree_id": "c65670098af5e5182be103e9239dfed3757664eb",
          "url": "https://github.com/maidsafe/safe_network/commit/d632a2d4831ffcbc8394fc7bb26e9b40008e1a8e"
        },
        "date": 1650439097688,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10529476647,
            "range": "± 8124611511",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3855001687,
            "range": "± 837795195",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 9573016532,
            "range": "± 285852598",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10037372219,
            "range": "± 2954800606",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3167152649,
            "range": "± 722786540",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4719129712,
            "range": "± 231249129",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "chriso83@protonmail.com",
            "name": "Chris O'Neil",
            "username": "jacderida"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "6452690c1b75bb8804c1f9de19c394a83f178acb",
          "message": "chore: remove modules that only contained tests\n\nDue to refactoring the issue tracking into a single `track_issue` function, these modules didn't end\nup having any code, just tests.\n\nThe tests were moved to separate testing modules in the `detection` module.",
          "timestamp": "2022-04-20T10:32:42+02:00",
          "tree_id": "d9a7d16c5666f16a0e0d3a0e7fc406c24d3b75b0",
          "url": "https://github.com/maidsafe/safe_network/commit/6452690c1b75bb8804c1f9de19c394a83f178acb"
        },
        "date": 1650445396739,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10494596035,
            "range": "± 8084147238",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3978267042,
            "range": "± 945710299",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 8557125025,
            "range": "± 6984369880",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10035096520,
            "range": "± 8345067",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3547740943,
            "range": "± 941933001",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4480761774,
            "range": "± 244529210",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joshuef@gmail.com",
            "name": "Josh Wilson",
            "username": "joshuef"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "aef7713600a5d036da90b771275dd60a661a4fe3",
          "message": "ci: add in specific no-run test cpmliation for unit tests.\n\nNode tests had become unreliable without this due to compilation noise",
          "timestamp": "2022-04-20T10:39:18+02:00",
          "tree_id": "cc7d79c29be431cb231d31ed403945c180bf80b1",
          "url": "https://github.com/maidsafe/safe_network/commit/aef7713600a5d036da90b771275dd60a661a4fe3"
        },
        "date": 1650445463094,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10486228205,
            "range": "± 9005841137",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3793030853,
            "range": "± 754323298",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 8828692240,
            "range": "± 1136397309",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10039633592,
            "range": "± 2217798973",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3165093581,
            "range": "± 960563532",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4490786204,
            "range": "± 187766539",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joshuef@gmail.com",
            "name": "Josh Wilson",
            "username": "joshuef"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": false,
          "id": "4b74a904c06c322e1d48968c4eed5aa25b7ab1b3",
          "message": "ci: remove sn working dir from droplet run after sn_client/node refactor",
          "timestamp": "2022-04-20T10:45:03+02:00",
          "tree_id": "ed6c56fdd499619bdfaf5a77d5fdf3b4038f5b43",
          "url": "https://github.com/maidsafe/safe_network/commit/4b74a904c06c322e1d48968c4eed5aa25b7ab1b3"
        },
        "date": 1650445788001,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10573296116,
            "range": "± 9799845229",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3747624375,
            "range": "± 980434474",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 8596236383,
            "range": "± 300438537",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10035701350,
            "range": "± 5248485",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3148911677,
            "range": "± 978103575",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4588752525,
            "range": "± 912520236",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "f359a45971a5b42a6f174536475f47b8ab076901",
          "message": "Merge #1122\n\n1122: Chore ci improvement r=Yoga07 a=joshuef\n\n<!--\r\nThanks for contributing to the project! We recommend you check out our \"Guide to contributing\" page if you haven't already: https://github.com/maidsafe/QA/blob/master/CONTRIBUTING.md\r\n\r\nWrite your comment below this line: -->\r\n\n\nCo-authored-by: Josh Wilson <joshuef@gmail.com>",
          "timestamp": "2022-04-20T08:45:06Z",
          "tree_id": "ed6c56fdd499619bdfaf5a77d5fdf3b4038f5b43",
          "url": "https://github.com/maidsafe/safe_network/commit/f359a45971a5b42a6f174536475f47b8ab076901"
        },
        "date": 1650449645892,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10498393056,
            "range": "± 7783065459",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3935521986,
            "range": "± 4001081089",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 9811635739,
            "range": "± 11655056084",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 7712974227,
            "range": "± 1129002141",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 4323571368,
            "range": "± 1208401645",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4828642227,
            "range": "± 946193061",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joshuef@gmail.com",
            "name": "Josh Wilson",
            "username": "joshuef"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "190f90bab927ba8269607a2686468ec7e80c303c",
          "message": "ci: droplet run now uses sn_client instead of safe_network package for client tests",
          "timestamp": "2022-04-20T14:47:46+02:00",
          "tree_id": "92bae442d7d9041bf47e0119e558ea238b683f12",
          "url": "https://github.com/maidsafe/safe_network/commit/190f90bab927ba8269607a2686468ec7e80c303c"
        },
        "date": 1650460483549,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10502987626,
            "range": "± 6775973770",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3993618652,
            "range": "± 1651674915",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 8776262617,
            "range": "± 6046328090",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 7702590074,
            "range": "± 1841441627",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3145666264,
            "range": "± 735370358",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4583993027,
            "range": "± 1273275020",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "gabrielviganotti@gmail.com",
            "name": "bochaco",
            "username": "bochaco"
          },
          "committer": {
            "email": "gabrielviganotti@gmail.com",
            "name": "bochaco",
            "username": "bochaco"
          },
          "distinct": true,
          "id": "91606f631a211d959364cab1e428d1ac895d3dca",
          "message": "tests(api): additional wallet API test cases",
          "timestamp": "2022-04-21T19:21:57-03:00",
          "tree_id": "43e0f954ad0d7c6dfd2c29b422208b83c2352f37",
          "url": "https://github.com/maidsafe/safe_network/commit/91606f631a211d959364cab1e428d1ac895d3dca"
        },
        "date": 1650581727669,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10502402781,
            "range": "± 6006595427",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3809691389,
            "range": "± 747117854",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 8609614520,
            "range": "± 231502075",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10036301721,
            "range": "± 6874162",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3154208627,
            "range": "± 973128336",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4444463406,
            "range": "± 355608883",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "85e02c9629d2547ab915202ff16ac50844d27ced",
          "message": "Merge #1105\n\n1105: feat(api): first and basic implementation of Wallet reissue API and related CLI cmds r=jacderida a=bochaco\n\nThis is a follow up PR (2nd PR) to PR #1097 , introducing first and basic implementation of Wallet reissue API and CLI cmd.\r\n- Reissue an output DBC from a Wallet (using [sn_dbc::TransactionBuilder](https://github.com/maidsafe/sn_dbc/blob/d275810f33376acc34cf7fd3f8045f585b264791/src/builder.rs#L36)) for a provided amount\r\n- Reissue the change DBC which is automatically stored in the source Wallet the reissue was made from\r\n- Spent DBCs are automatically soft-removed from the source Wallet (Multimap)\r\n- Reissued DBCs are all bearer at this instance\r\n\r\nThe CLI `wallet reissue` and `wallet deposit` commands are currently compatible with the DBC's generated and printed out by the [mint-repl](https://github.com/maidsafe/sn_dbc#mint-repl-example) and [DBC Playground](https://github.com/maidsafe/sn_dbc_examples) output, once a DBC is generated by those example apps, it can be deposited in a Wallet and check its balance, e.g. the following commands deposit the genesis DBC into a Wallet on Safe, check its total balance, and reissue a DBC from the Wallet for 3.77 safecoins:\r\n```\r\n$ safe wallet create\r\nWallet created at: \"safe://hyryynyen8rzu4umhifw6j7chbr5wthr1osfb3qcc78n1498zrpd8wsm4y5my84y\"\r\n\r\n$ safe wallet balance safe://hyryynyen8rzu4umhifw6j7chbr5wthr1osfb3qcc78n1498zrpd8wsm4y5my84y\r\nWallet at \"safe://hyryynyen8rzu4umhifw6j7chbr5wthr1osfb3qcc78n1498zrpd8wsm4y5my84y\" has a total balance of 0.000000000 safecoins\r\n\r\n$ safe wallet deposit --name my-first-dbc safe://hyryynyen8rzu4umhifw6j7chbr5wthr1osfb3qcc78n1498zrpd8wsm4y5my84y c4a1953dfa419234b8ab9a...\r\nSpendable DBC deposited with name 'my-first-dbc' in Wallet located at \"safe://hyryynyen8rzu4umhifw6j7chbr5wthr1osfb3qcc78n1498zrpd8wsm4y5my84y\"\r\n\r\n$ safe wallet balance safe://hyryynyen8rzu4umhifw6j7chbr5wthr1osfb3qcc78n1498zrpd8wsm4y5my84y\r\nWallet at \"safe://hyryynyen8rzu4umhifw6j7chbr5wthr1osfb3qcc78n1498zrpd8wsm4y5my84y\" has a total balance of 18446744073.709551615 safecoins\r\n\r\n$ safe wallet reissue 3.77 --from safe://hyryynyen8rzu4umhifw6j7chbr5wthr1osfb3qcc78n1498zrpd8wsm4y5my84y\r\nSuccess. Reissued DBC with 3.77 safecoins:\r\n-------- DBC DATA --------\r\n0000000000000000d3dfd5b56aeea1b906c82.......bb11e30f382dda25700000000\r\n--------------------------\r\n\r\n$ safe wallet balance safe://hyryynyen8rzu4umhifw6j7chbr5wthr1osfb3qcc78n1498zrpd8wsm4y5my84y\r\nWallet at \"safe://hyryynyen8rzu4umhifw6j7chbr5wthr1osfb3qcc78n1498zrpd8wsm4y5my84y\" has a total balance of 18446744069.939551615 safecoins\r\n```\r\n\r\nAs next steps to work on in separate PRs:\r\n- Verification of generated Tx and spentproofs upon reissuing DBCs (depends on spentbook implementation and messaging to be available)\r\n- Log input DBCs as spent on the network's spentbook upon reissuing DBCs (depends on spentbook implementation and messaging to be available)\r\n- Verification of spentproofs upon depositing DBCs in Wallets (depends on spentbook implementation and messaging to be available)\r\n- Use of input decoys in the transactions upon reissuing DBCs\r\n- Support for non-bearer DBCs\r\n- Review how exactly we want to serialise and store the Wallets, currently using Private Register+Chunks\r\n- The sn-api unit tests can be improved if test DBCs can be generated with different amounts. Generate test DBCs from an amount and SK instead of deserialising a hard-coded serialised DBC. Hard-coded serialised DBCs are currently generated with sn_dbc mint-repl example.\n\nCo-authored-by: bochaco <gabrielviganotti@gmail.com>",
          "timestamp": "2022-04-21T22:22:24Z",
          "tree_id": "43e0f954ad0d7c6dfd2c29b422208b83c2352f37",
          "url": "https://github.com/maidsafe/safe_network/commit/85e02c9629d2547ab915202ff16ac50844d27ced"
        },
        "date": 1650585076321,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10485016102,
            "range": "± 6100684336",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 4035700607,
            "range": "± 1226468729",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 8631868729,
            "range": "± 1222312314",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10037227655,
            "range": "± 985717478",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3147424934,
            "range": "± 27027014",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4496267462,
            "range": "± 1127609452",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "chriso83@protonmail.com",
            "name": "Chris O'Neil",
            "username": "jacderida"
          },
          "committer": {
            "email": "chris.oneil@gmail.com",
            "name": "Chris O'Neil",
            "username": "jacderida"
          },
          "distinct": true,
          "id": "ae4ee5c1a37dab7b5ca132d96d026bcdbac50be0",
          "message": "fix: use supported referencing style\n\nCurrently smart-release doesn't support the `~` style of reference; the `^` style must be used. This\ncaused the last nightly run to fail at version bumping.",
          "timestamp": "2022-04-22T17:46:13+01:00",
          "tree_id": "d215a76d9ef4f65758161998049a8bee598cdaea",
          "url": "https://github.com/maidsafe/safe_network/commit/ae4ee5c1a37dab7b5ca132d96d026bcdbac50be0"
        },
        "date": 1650647526695,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10582101859,
            "range": "± 11266329639",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3802376253,
            "range": "± 1540926960",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 9078344562,
            "range": "± 6634695919",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10036456800,
            "range": "± 2216565963",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3145873460,
            "range": "± 972589923",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4548920785,
            "range": "± 1456535226",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "chriso83@protonmail.com",
            "name": "Chris O'Neil",
            "username": "jacderida"
          },
          "committer": {
            "email": "chris.oneil@gmail.com",
            "name": "Chris O'Neil",
            "username": "jacderida"
          },
          "distinct": true,
          "id": "a477c1db40b9d8f78adf3f620942a06daf0ecc2b",
          "message": "ci: incorporate sn_client and sn_node in release process\n\nThe `sn_client` and `sn_node` crates had to be included for publishing in the release process.\n\nThere were a few other changes I made to support this:\n\n* The title of the release, with all the crate names, was getting too large. I changed it to just\n  include the version numbers. The description of the release now includes the list of crates and\n  the version numbers they relate to.\n* Stop passing the version numbers around for the changelog generation. We can just read them from\n  the Cargo manifest.\n* Change crate publishing to a sequential process, rather than have different jobs.",
          "timestamp": "2022-04-23T02:07:59+01:00",
          "tree_id": "25e73d412465a502cc619bc40938d235de4487e3",
          "url": "https://github.com/maidsafe/safe_network/commit/a477c1db40b9d8f78adf3f620942a06daf0ecc2b"
        },
        "date": 1650677712391,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10664191688,
            "range": "± 9312596260",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 4105387853,
            "range": "± 4970783799",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 9648606681,
            "range": "± 264164836",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10037757204,
            "range": "± 2950944439",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3175404074,
            "range": "± 1113880210",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4715985218,
            "range": "± 7051399492",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "action@github.com",
            "name": "GitHub Action",
            "username": "actions-user"
          },
          "committer": {
            "email": "action@github.com",
            "name": "GitHub Action",
            "username": "actions-user"
          },
          "distinct": true,
          "id": "2f4e7e6305ba387f2e28945aee71df650ac1d3eb",
          "message": "chore(release): sn_interface-0.2.0/sn_dysfunction-0.1.2/sn_api-0.59.0/sn_cli-0.52.0",
          "timestamp": "2022-04-23T02:41:20Z",
          "tree_id": "3f62f7bb9746c33c6ce36ccebd6c1eac2ebd94a6",
          "url": "https://github.com/maidsafe/safe_network/commit/2f4e7e6305ba387f2e28945aee71df650ac1d3eb"
        },
        "date": 1650683216123,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10587230086,
            "range": "± 6199645031",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3999300356,
            "range": "± 1943761283",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 8773638969,
            "range": "± 6168197221",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10035740201,
            "range": "± 2215638661",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3157920581,
            "range": "± 1117066788",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4497098562,
            "range": "± 102181609",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "e49d38239b3a8c468616ad3782e1208316e9b5e0",
          "message": "Merge #1128\n\n1128: tests(resource_proof): test valid nonce signature r=Yoga07 a=RolandSherwin\n\nMakes sure that the `nonce_signature` is signed by the correct peer.\n\nCo-authored-by: RolandSherwin <RolandSherwin@protonmail.com>",
          "timestamp": "2022-04-25T07:23:32Z",
          "tree_id": "b944a6ca44683c5de702a72c1ab3f88589e30fd8",
          "url": "https://github.com/maidsafe/safe_network/commit/e49d38239b3a8c468616ad3782e1208316e9b5e0"
        },
        "date": 1650877065314,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10485377510,
            "range": "± 10631617299",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3767715033,
            "range": "± 730171525",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 8500721472,
            "range": "± 3687151215",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10033546845,
            "range": "± 2953869458",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3150609602,
            "range": "± 14045994",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4203737346,
            "range": "± 202317743",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joshuef@gmail.com",
            "name": "Josh Wilson",
            "username": "joshuef"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "1e7c4ab6d56304f99d11396e0eee5109eb4dda04",
          "message": "fix: update some instances of safe_network->sn_node",
          "timestamp": "2022-04-25T10:49:52+02:00",
          "tree_id": "3ff1ead01548cdb78744fd85471b961f852b7b0f",
          "url": "https://github.com/maidsafe/safe_network/commit/1e7c4ab6d56304f99d11396e0eee5109eb4dda04"
        },
        "date": 1650878925631,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10537229523,
            "range": "± 10933435611",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3886009950,
            "range": "± 49337242",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 10566585871,
            "range": "± 1878107501",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10042458842,
            "range": "± 2213508850",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3174226841,
            "range": "± 20377604",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4888405804,
            "range": "± 11860645046",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joshuef@gmail.com",
            "name": "Josh Wilson",
            "username": "joshuef"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "9aa65d92e1d806150401f8bdefa1ead2e3cafd42",
          "message": "fix: use the config verbosity if no env var present",
          "timestamp": "2022-04-25T11:16:30+02:00",
          "tree_id": "0aa332d8e951b1212b14d7575af1a9f058bb7d07",
          "url": "https://github.com/maidsafe/safe_network/commit/9aa65d92e1d806150401f8bdefa1ead2e3cafd42"
        },
        "date": 1650879740800,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10482595552,
            "range": "± 9891582554",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3817879863,
            "range": "± 4159423845",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 8863759835,
            "range": "± 1069985635",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10034899575,
            "range": "± 2955725651",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3180271163,
            "range": "± 1192732830",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4626292110,
            "range": "± 954409409",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "action@github.com",
            "name": "GitHub Action",
            "username": "actions-user"
          },
          "committer": {
            "email": "action@github.com",
            "name": "GitHub Action",
            "username": "actions-user"
          },
          "distinct": true,
          "id": "cbf5d45ec4522961fc7ef0860d86cc7d5e0ecca8",
          "message": "chore(release): sn_node-0.58.14",
          "timestamp": "2022-04-25T10:58:08Z",
          "tree_id": "b07ec67ce1be245303bcae9eb0f16c727cdd693d",
          "url": "https://github.com/maidsafe/safe_network/commit/cbf5d45ec4522961fc7ef0860d86cc7d5e0ecca8"
        },
        "date": 1650885943609,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10489626796,
            "range": "± 2177525503",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3958794073,
            "range": "± 690517635",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 9858483285,
            "range": "± 2196104359",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 7718447719,
            "range": "± 1574537581",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3207496824,
            "range": "± 1102149675",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4716450570,
            "range": "± 1008330688",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "gabrielviganotti@gmail.com",
            "name": "bochaco",
            "username": "bochaco"
          },
          "committer": {
            "email": "gabrielviganotti@gmail.com",
            "name": "bochaco",
            "username": "bochaco"
          },
          "distinct": true,
          "id": "3894e8ed5ab48bc72287c4ae74fa53ef0ba51aaa",
          "message": "chore(sn_node): remove the max-capacity flag from sn_node cli",
          "timestamp": "2022-04-26T05:28:30-03:00",
          "tree_id": "befc30b29df210109de5c0bb373a91cbd9daf886",
          "url": "https://github.com/maidsafe/safe_network/commit/3894e8ed5ab48bc72287c4ae74fa53ef0ba51aaa"
        },
        "date": 1650963698754,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10503466365,
            "range": "± 8836034545",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3766689173,
            "range": "± 108417435",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 8771217777,
            "range": "± 12358603919",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10036808108,
            "range": "± 2217226671",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3151501116,
            "range": "± 731064419",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4538416797,
            "range": "± 185847986",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "3448a0b6f1b6278dbbf8a177714a8c3b6b3832cd",
          "message": "Merge #1137\n\n1137: chore(node): change default node max cpacity to 10GB r=joshuef a=bochaco\n\n- This also removes some outdated warning message shown by CLI.\n\nCo-authored-by: Southside <293741+willief@users.noreply.github.com>\nCo-authored-by: bochaco <gabrielviganotti@gmail.com>",
          "timestamp": "2022-04-26T08:28:48Z",
          "tree_id": "befc30b29df210109de5c0bb373a91cbd9daf886",
          "url": "https://github.com/maidsafe/safe_network/commit/3448a0b6f1b6278dbbf8a177714a8c3b6b3832cd"
        },
        "date": 1650966933035,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10667310958,
            "range": "± 11973296994",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3792752587,
            "range": "± 1203560877",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 8806514945,
            "range": "± 5633849568",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10039529258,
            "range": "± 6214681",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3149400062,
            "range": "± 740383351",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4576938875,
            "range": "± 1033440617",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "22abbc73f909131a0208ddc6e9471d073061134a",
          "message": "Merge #1139\n\n1139: chore(messaging): rename MsgKind -> AuthKind r=Yoga07 a=joshuef\n\nThis feels more correct given that the kind is actually about the authority that\r\nthe message carries.\r\n\r\n<!--\r\nThanks for contributing to the project! We recommend you check out our \"Guide to contributing\" page if you haven't already: https://github.com/maidsafe/QA/blob/master/CONTRIBUTING.md\r\n\r\nWrite your comment below this line: -->\r\n\n\nCo-authored-by: Josh Wilson <joshuef@gmail.com>\nCo-authored-by: David Irvine <david.irvine@maidsafe.net>\nCo-authored-by: Yogeshwar Murugan <yogeshwar_1997@hotmail.com>",
          "timestamp": "2022-04-26T10:19:19Z",
          "tree_id": "baa61c65d0977bdece8e06378e1c36b30fe05c55",
          "url": "https://github.com/maidsafe/safe_network/commit/22abbc73f909131a0208ddc6e9471d073061134a"
        },
        "date": 1650973556932,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10479742010,
            "range": "± 7656780972",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3801157716,
            "range": "± 1119595555",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 8851481645,
            "range": "± 1299509327",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10038800917,
            "range": "± 5087920",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3145647318,
            "range": "± 1206656934",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4499987441,
            "range": "± 1389669111",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "ee439c13b889a342247bcc5ab9ff62ba2f67a591",
          "message": "Merge #1138\n\n1138: Feat interface service flag r=joshuef a=joshuef\n\n<!--\r\nThanks for contributing to the project! We recommend you check out our \"Guide to contributing\" page if you haven't already: https://github.com/maidsafe/QA/blob/master/CONTRIBUTING.md\r\n\r\nWrite your comment below this line: -->\r\n\n\nCo-authored-by: Josh Wilson <joshuef@gmail.com>",
          "timestamp": "2022-04-26T13:18:57Z",
          "tree_id": "05980294b460730c2f6378e0c87ce5f3952b1e52",
          "url": "https://github.com/maidsafe/safe_network/commit/ee439c13b889a342247bcc5ab9ff62ba2f67a591"
        },
        "date": 1650984341415,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10509118574,
            "range": "± 12114263182",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3924128246,
            "range": "± 4457097572",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 9407370887,
            "range": "± 1857495435",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10038604331,
            "range": "± 10242538",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3148867854,
            "range": "± 760184730",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4623495149,
            "range": "± 90805765",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "865f24477244155528583afa5a3655690e4b7093",
          "message": "Merge #1141\n\n1141: ci: increase alert threshold r=joshuef a=joshuef\n\nwe were seeing fails when no related code was touched,\r\nthis should hopefully keep that noise from affecting the CI too badly\r\n\r\n<!--\r\nThanks for contributing to the project! We recommend you check out our \"Guide to contributing\" page if you haven't already: https://github.com/maidsafe/QA/blob/master/CONTRIBUTING.md\r\n\r\nWrite your comment below this line: -->\r\n\n\nCo-authored-by: Josh Wilson <joshuef@gmail.com>",
          "timestamp": "2022-04-26T14:20:29Z",
          "tree_id": "0691d80ba555eaf4e7fca454486589d71309c0d4",
          "url": "https://github.com/maidsafe/safe_network/commit/865f24477244155528583afa5a3655690e4b7093"
        },
        "date": 1650988384870,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 20449916330,
            "range": "± 13669585595",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3815633901,
            "range": "± 5312799349",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 8568977611,
            "range": "± 2133441476",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10037256272,
            "range": "± 1127908042",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3160619048,
            "range": "± 1565871175",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4761627136,
            "range": "± 1422021445",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joshuef@gmail.com",
            "name": "Josh Wilson",
            "username": "joshuef"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "a4b7597853c9f154e6fd04f1f82133cab0b3c784",
          "message": "fix: add missing backpressure feature gate.\n\nWe were trying to count messages when thsi wasn't instantiated w/o\nbackpressure. So were logging a looot of errors.",
          "timestamp": "2022-04-27T14:18:36+02:00",
          "tree_id": "38f4e31fee579bf274911899711e209c6db53da6",
          "url": "https://github.com/maidsafe/safe_network/commit/a4b7597853c9f154e6fd04f1f82133cab0b3c784"
        },
        "date": 1651063926385,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10495913132,
            "range": "± 12263123638",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3782330805,
            "range": "± 996333405",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 8745720287,
            "range": "± 312160251",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10035592269,
            "range": "± 3382477047",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3138213047,
            "range": "± 15366773",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4473295167,
            "range": "± 139860963",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "b4c8086a53d20c588b4c4c941601edd3f360e04b",
          "message": "Merge #1142\n\n1142: feat(api): return a Token value from wallet balance API instead of a string r=joshuef a=bochaco\n\n- Additionally add support to the `cat` and `dog` CLI commands for `Wallets`.\r\n- When cat command is used with a `Wallet`, it lists all spendable balances found in it,\r\nas long as the user has permissions to read it as it's expected to be a private `Multimap`.\r\n\r\nBREAKING CHANGE: `wallet_balance` API return type has been changed.\r\n\r\nResolves #1110 \r\n\r\nExample outputs for `cat` ad `dog` on a Wallet:\r\n```\r\n$ safe cat safe://hyryynyenfnx888i7xdyu6ukykb9mu9fpek8y35t8m1huohzp6rgmudxg31jy84y\r\nSpendable balances of Wallet at \"safe://hyryynyenfnx888i7xdyu6ukykb9mu9fpek8y35t8m1huohzp6rgmudxg31jy84y\":\r\n+------------------------------------------------------------------+-------------+---------------------+\r\n| Spendable balance name                                           | Balance     | DBC Data            |\r\n|------------------------------------------------------------------+-------------+---------------------|\r\n| a02a566c75aa1140aacb118fffd305298756d369e034e2caa9ccfa3e3ed56231 | 0.180000000 | 8dda5h14...00000000 |\r\n|------------------------------------------------------------------+-------------+---------------------|\r\n| my-other-dbc                                                     | 1.530000000 | 2fce3f70...00000000 |\r\n+------------------------------------------------------------------+-------------+---------------------+\r\n\r\n$ safe dog safe://hyryynyenfnx888i7xdyu6ukykb9mu9fpek8y35t8m1huohzp6rgmudxg31jy84y\r\n\r\n== URL resolution step 1 ==\r\nResolved from: safe://hyryynyenfnx888i7xdyu6ukykb9mu9fpek8y35t8m1huohzp6rgmudxg31jy84y\r\n= Wallet =\r\nXOR-URL: safe://hyryynyenfnx888i7xdyu6ukykb9mu9fpek8y35t8m1huohzp6rgmudxg31jy84y\r\nType tag: 1000\r\nXOR name: 0x289e739ebd78c13f4d40507eb9fcad428e0cee275cb93872edf10cb98de6cc92\r\nNative data type: Private Register\r\n```\n\nCo-authored-by: bochaco <gabrielviganotti@gmail.com>",
          "timestamp": "2022-04-27T12:19:53Z",
          "tree_id": "8b0c504193057feffe9283eddfabba69798c985d",
          "url": "https://github.com/maidsafe/safe_network/commit/b4c8086a53d20c588b4c4c941601edd3f360e04b"
        },
        "date": 1651067161971,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10501437118,
            "range": "± 6732916370",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3962047183,
            "range": "± 69841963",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 10048125813,
            "range": "± 6410596738",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10039661116,
            "range": "± 3379678538",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3177774630,
            "range": "± 717264286",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4886676210,
            "range": "± 1279579925",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "7058ecce9a1f9ca90c353a8f0705d81aad8943a2",
          "message": "Merge #1149\n\n1149: fix(membership): avoid AE loop by being judicious with AE requests r=joshuef a=davidrusu\n\nThe membership was requesting AE whenever we hit an error when processing a vote. This can lead to an infinite loop if the error is not related to lacking information:\r\n\r\n1. node A sends bad vote to B.\r\n2. B processes bad vote and returns error and B requests AE from A\r\n3. node A processes the AE request and re-sends the bad vote to B\r\n4. goto 1.\r\n\r\nFix is to only request AE when we are in the wrong generation.\n\nCo-authored-by: David Rusu <davidrusu.me@gmail.com>",
          "timestamp": "2022-04-28T07:30:14Z",
          "tree_id": "8639522f1ba556e90ca0977afa4f52be93a68e88",
          "url": "https://github.com/maidsafe/safe_network/commit/7058ecce9a1f9ca90c353a8f0705d81aad8943a2"
        },
        "date": 1651136852235,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10672520221,
            "range": "± 10752493566",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3753572436,
            "range": "± 754020304",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 8305744692,
            "range": "± 401181991",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10037981703,
            "range": "± 2217426136",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3144133606,
            "range": "± 2199849337",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4626326874,
            "range": "± 1277022537",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joshuef@gmail.com",
            "name": "Josh Wilson",
            "username": "joshuef"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "155d62257546868513627709742215c0c8f9574f",
          "message": "chore(sn_interface): check and log for shrinking SAP on verify_with_chain",
          "timestamp": "2022-05-03T09:22:28+02:00",
          "tree_id": "edde8b4861c04b438e0a10160ce5f08a96775f2b",
          "url": "https://github.com/maidsafe/safe_network/commit/155d62257546868513627709742215c0c8f9574f"
        },
        "date": 1651564567907,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10498378947,
            "range": "± 9509755235",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3753554952,
            "range": "± 715812592",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 8379258982,
            "range": "± 885377313",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10036716175,
            "range": "± 4914731",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3148452379,
            "range": "± 737723215",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4496980603,
            "range": "± 1420453791",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "d46e85bf508be983017b90e6ce18f588039b16ac",
          "message": "Merge #1160\n\n1160: Chore increase client knowledge and validation of AE messages r=joshuef a=joshuef\n\n<!--\r\nThanks for contributing to the project! We recommend you check out our \"Guide to contributing\" page if you haven't already: https://github.com/maidsafe/QA/blob/master/CONTRIBUTING.md\r\n\r\nWrite your comment below this line: -->\r\n\n\nCo-authored-by: Josh Wilson <joshuef@gmail.com>",
          "timestamp": "2022-05-04T13:12:12Z",
          "tree_id": "097a67e69c0db4e32fa847f6fb35d25e34df8dd4",
          "url": "https://github.com/maidsafe/safe_network/commit/d46e85bf508be983017b90e6ce18f588039b16ac"
        },
        "date": 1651675721955,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10504232892,
            "range": "± 6959020517",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3826628750,
            "range": "± 1126521596",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 9068567497,
            "range": "± 160696616",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10037728325,
            "range": "± 2954871130",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3142446588,
            "range": "± 986563369",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4625007382,
            "range": "± 1069709067",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "chriso83@protonmail.com",
            "name": "Chris O'Neil",
            "username": "jacderida"
          },
          "committer": {
            "email": "chris.oneil@gmail.com",
            "name": "Chris O'Neil",
            "username": "jacderida"
          },
          "distinct": true,
          "id": "a98d574b470a5df60ff1ff7c9112b1e6690b34e0",
          "message": "chore: use the nextest action from maidsafe org\n\nI transferred this action from my own personal namespace to the `maidsafe` organisation.",
          "timestamp": "2022-05-04T21:22:41+01:00",
          "tree_id": "ada93f15ba12125acba789c5b5218f235ac5934b",
          "url": "https://github.com/maidsafe/safe_network/commit/a98d574b470a5df60ff1ff7c9112b1e6690b34e0"
        },
        "date": 1651697406306,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10490261051,
            "range": "± 2894680593",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 4081159613,
            "range": "± 6536988608",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 9355073366,
            "range": "± 323948240",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10044509610,
            "range": "± 2216642064",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3206171800,
            "range": "± 1180008161",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4744827709,
            "range": "± 237954335",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "a4c5ccb8bb7fbbf8ab4052d3b1051f8cac100d53",
          "message": "Merge #1162\n\n1162: Membership ae fixes r=joshuef a=joshuef\n\n<!--\r\nThanks for contributing to the project! We recommend you check out our \"Guide to contributing\" page if you haven't already: https://github.com/maidsafe/QA/blob/master/CONTRIBUTING.md\r\n\r\nWrite your comment below this line: -->\r\n\n\nCo-authored-by: Josh Wilson <joshuef@gmail.com>\nCo-authored-by: davidrusu <davidrusu.me@gmail.com>",
          "timestamp": "2022-05-05T18:22:29Z",
          "tree_id": "75e33d8f2f41781fa814a2e105068b4761578273",
          "url": "https://github.com/maidsafe/safe_network/commit/a4c5ccb8bb7fbbf8ab4052d3b1051f8cac100d53"
        },
        "date": 1651780563349,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10502296186,
            "range": "± 136161682",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3794837591,
            "range": "± 734713668",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 8677869394,
            "range": "± 2061688228",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10032639246,
            "range": "± 2953111731",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3156299528,
            "range": "± 722852929",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4520236870,
            "range": "± 999336604",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "action@github.com",
            "name": "GitHub Action",
            "username": "actions-user"
          },
          "committer": {
            "email": "action@github.com",
            "name": "GitHub Action",
            "username": "actions-user"
          },
          "distinct": true,
          "id": "737d906a61f772593ac7df755d995d66059e8b5e",
          "message": "chore(release): sn_interface-0.2.1/sn_client-0.62.1/sn_node-0.58.15/sn_api-0.60.0/sn_cli-0.53.0",
          "timestamp": "2022-05-06T21:14:49Z",
          "tree_id": "21b82fae67c5be54bf517f244f35aaedeaa20dd0",
          "url": "https://github.com/maidsafe/safe_network/commit/737d906a61f772593ac7df755d995d66059e8b5e"
        },
        "date": 1651873808188,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10486443115,
            "range": "± 13201622551",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3836199806,
            "range": "± 1482257831",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 9248790410,
            "range": "± 2108900540",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10037419424,
            "range": "± 2218072712",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3139801680,
            "range": "± 738472448",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4779707166,
            "range": "± 1224031352",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "e5d0c17c335a3a25ee0bb4c81906fa176abeb7f5",
          "message": "Merge #1169\n\n1169: chore: add ProposalAgreed log marker r=Yoga07 a=joshuef\n\n<!--\r\nThanks for contributing to the project! We recommend you check out our \"Guide to contributing\" page if you haven't already: https://github.com/maidsafe/QA/blob/master/CONTRIBUTING.md\r\n\r\nWrite your comment below this line: -->\r\n\n\nCo-authored-by: Josh Wilson <joshuef@gmail.com>",
          "timestamp": "2022-05-09T07:58:30Z",
          "tree_id": "2f66e692b2b2bb5f5ad52fcfb61dcfbd740c71ec",
          "url": "https://github.com/maidsafe/safe_network/commit/e5d0c17c335a3a25ee0bb4c81906fa176abeb7f5"
        },
        "date": 1652089665772,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 16752616507,
            "range": "± 5018765869",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3730685055,
            "range": "± 996593540",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 8194560938,
            "range": "± 362287744",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10037655275,
            "range": "± 3977526",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3131072028,
            "range": "± 1571056060",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4430397995,
            "range": "± 1010859547",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "5b21c663c7f11124f0ed2f330b2f8687745f7da7",
          "message": "Merge #1167\n\n1167: docs: Add recursive flag to rm of dir r=joshuef a=dirvine\n\nIn addition removed unneeded map_err\r\n\r\n<!--\r\nThanks for contributing to the project! We recommend you check out our \"Guide to contributing\" page if you haven't already: https://github.com/maidsafe/QA/blob/master/CONTRIBUTING.md\r\n\r\nWrite your comment below this line: -->\r\n\n\nCo-authored-by: David Irvine <david.irvine@maidsafe.net>\nCo-authored-by: joshuef <joshuef@gmail.com>",
          "timestamp": "2022-05-09T11:40:04Z",
          "tree_id": "5ca12db92dc66ea452ace1acf796864f01cb1f16",
          "url": "https://github.com/maidsafe/safe_network/commit/5b21c663c7f11124f0ed2f330b2f8687745f7da7"
        },
        "date": 1652102096367,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10493047304,
            "range": "± 8524835117",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 4126703099,
            "range": "± 1635257473",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 9605267496,
            "range": "± 13864268660",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10043973767,
            "range": "± 2956926000",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3152105020,
            "range": "± 32411713",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4690686941,
            "range": "± 1009966132",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "e08096d37dfab490f22ae9786a006aa3f9f630c1",
          "message": "Merge #1165\n\n1165: chore: use different retry count for certain runs r=joshuef a=jacderida\n\nAlso use Cargo to install ripgrep, which is a cross-platform solution.\n\nCo-authored-by: Chris O'Neil <chriso83@protonmail.com>\nCo-authored-by: joshuef <joshuef@gmail.com>",
          "timestamp": "2022-05-09T13:49:50Z",
          "tree_id": "6b90ca7ab26e92e06b34d4c77de2f2458a1650c4",
          "url": "https://github.com/maidsafe/safe_network/commit/e08096d37dfab490f22ae9786a006aa3f9f630c1"
        },
        "date": 1652109368679,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10520126387,
            "range": "± 6533345739",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3902796723,
            "range": "± 1457007505",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 9693359830,
            "range": "± 334816087",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10041569016,
            "range": "± 7638326",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3147131549,
            "range": "± 31587400",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4935858107,
            "range": "± 1438321306",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "459b641f22b488f33825777b974da80512eabed5",
          "message": "Merge #1140\n\n1140: tests(storage): model-based test for DataStorage module r=Yoga07 a=RolandSherwin\n\nModel based testing using the following logic:\r\n- Create random combinations of Store, Get, Query, Remove operations using `quickcheck` lib\r\n- Do the operations on `DataStorage` module as well as a `HashMap` and make sure that their results don't diverge.\r\n\r\nFew shortcomings:\r\n1. Cannot do non-blocking async calls because `quickcheck` does not support it (as far as I know). Found [this](https://github.com/nytopop/quickcheck_async) library, but was unsuccessful using it. Thus takes ~25 seconds to do 100 quickcheck tests on my system.\r\n2. Only implemented the tests for `ChunkStorage` field. \r\n3. The chunk sizes are in the range of 1 to 3 mb only.\n\nCo-authored-by: RolandSherwin <RolandSherwin@protonmail.com>",
          "timestamp": "2022-05-09T14:51:26Z",
          "tree_id": "9cdc91b857a36b76cd59e7b59e585a2610938edf",
          "url": "https://github.com/maidsafe/safe_network/commit/459b641f22b488f33825777b974da80512eabed5"
        },
        "date": 1652113162898,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10593333806,
            "range": "± 8852407539",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 4036137081,
            "range": "± 1635374867",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 9548970037,
            "range": "± 642767585",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10040813307,
            "range": "± 2955988103",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3234065827,
            "range": "± 1190979959",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4777427117,
            "range": "± 145064643",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "837c44cda38c2757f689cc4db4a84fa7c02091c0",
          "message": "Merge #1172\n\n1172: fix(client): check Register permissions on ops locally to prevent failures when broadcasted to the network r=joshuef a=bochaco\n\nResolve PR #836.\r\n\r\nNote that commands creating a Register don't fail as the network doesn't return any type of errors (by design) but only an ACK that the message was delivered to the section. However, we can perform checks locally on CRDT operations, like in the case of a Register operation, so any permissions issue is detected locally on the client side before the operation is sent/broadcasted to the network. This PR, does simply that, it makes sure the permissions are checked locally when building the Register CRDT operation.\n\nCo-authored-by: bochaco <gabrielviganotti@gmail.com>",
          "timestamp": "2022-05-09T17:56:28Z",
          "tree_id": "4c576055231830c75dcd52fe07d7e234938bf2d9",
          "url": "https://github.com/maidsafe/safe_network/commit/837c44cda38c2757f689cc4db4a84fa7c02091c0"
        },
        "date": 1652124315855,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10484901795,
            "range": "± 7242120255",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 4088398091,
            "range": "± 4605330023",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 9533703121,
            "range": "± 644309610",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 8872762095,
            "range": "± 1635874445",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3148096232,
            "range": "± 751728468",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4804536275,
            "range": "± 960125196",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "06b4433f199ba7c622ad57e767d80f58f0b50a69",
          "message": "Merge #1171\n\n1171: Ci remove query limit r=bochaco a=joshuef\n\n<!--\r\nThanks for contributing to the project! We recommend you check out our \"Guide to contributing\" page if you haven't already: https://github.com/maidsafe/QA/blob/master/CONTRIBUTING.md\r\n\r\nWrite your comment below this line: -->\r\n\n\nCo-authored-by: Josh Wilson <joshuef@gmail.com>",
          "timestamp": "2022-05-09T18:57:30Z",
          "tree_id": "bf0c9bb024d64efa3eef4ac859bf6f9880c073c4",
          "url": "https://github.com/maidsafe/safe_network/commit/06b4433f199ba7c622ad57e767d80f58f0b50a69"
        },
        "date": 1652128160892,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 26267066573,
            "range": "± 12810449985",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3901633744,
            "range": "± 2083041325",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 9853642887,
            "range": "± 3718498366",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10029713288,
            "range": "± 2214225530",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 10115008888,
            "range": "± 48001543",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 10392302004,
            "range": "± 41707748",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "9a8789e307fa09b9624a8602978f720d3dc9fc8b",
          "message": "Merge #1175\n\n1175: ci: nightly improvements and fix release process issues r=joshuef a=jacderida\n\nA few changes for things spotted in the last nightly/release run:\r\n\r\n* Use `usize::MAX` for max capacity on ARM/ARMv7. A change to use a max capacity of 10GB wouldn't\r\n  compile on these 32-bit architectures, since the value exceeded 2^32.\r\n* Exit on error if ARM builds fail. Even though compilation failed, the release process didn't\r\n  report an error for the failure. The outer process must be disabling the `set -e` effect.\r\n* During the publishing process, instruct `sn_node` to wait on `sn_interface` rather than\r\n  `sn_dysfunction`, since `sn_interface` is published immediately before `sn_node`. The last release\r\n  failed when it tried to publish `sn_node` because `sn_interface` wasn't available yet.\r\n* Use 30 nodes in the testnet for the nightly run.\r\n* Run the CLI test suite in parallel with the API and client tests. Previously we didn't try this\r\n  because we never knew if the network would handle the load.\r\n\n\nCo-authored-by: Chris O'Neil <chriso83@protonmail.com>\nCo-authored-by: joshuef <joshuef@gmail.com>",
          "timestamp": "2022-05-09T20:40:12Z",
          "tree_id": "d0c31a0f6dfcbe577bb9f8c27060fb129417fd04",
          "url": "https://github.com/maidsafe/safe_network/commit/9a8789e307fa09b9624a8602978f720d3dc9fc8b"
        },
        "date": 1652134003455,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10575214525,
            "range": "± 11926418052",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 4138593225,
            "range": "± 3450934037",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 8471964706,
            "range": "± 229455706",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10034535191,
            "range": "± 2215474479",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3158308832,
            "range": "± 962475624",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4534062594,
            "range": "± 194734415",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "action@github.com",
            "name": "GitHub Action",
            "username": "actions-user"
          },
          "committer": {
            "email": "action@github.com",
            "name": "GitHub Action",
            "username": "actions-user"
          },
          "distinct": true,
          "id": "61ba367c308a846cb3f1ae065b1fbbdfb85838e4",
          "message": "chore(release): sn_interface-0.2.2/sn_client-0.62.2/sn_node-0.58.16/sn_api-0.60.1",
          "timestamp": "2022-05-10T05:53:51Z",
          "tree_id": "2aaa69c876271b9def6798c7a8a7273e7ad67a09",
          "url": "https://github.com/maidsafe/safe_network/commit/61ba367c308a846cb3f1ae065b1fbbdfb85838e4"
        },
        "date": 1652163521711,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10589558716,
            "range": "± 10594146845",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3792862627,
            "range": "± 980788015",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 8701314095,
            "range": "± 184190913",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10039255792,
            "range": "± 2213028862",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3157694333,
            "range": "± 982673674",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4440160264,
            "range": "± 203800391",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joshuef@gmail.com",
            "name": "Josh Wilson",
            "username": "joshuef"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "ddb939d5831b2f0d66fa2e0954b62e5e22a3ee69",
          "message": "fix(dysfunction): relax dysfunction for knowledge and conn issues\n\nIncreases 10x the amount of conn or knowledge issues. We've been voting\noff nodes far too quickly, even on droplet testnets",
          "timestamp": "2022-05-10T09:33:04+02:00",
          "tree_id": "856773313992b8ca01796987ce18d00578e31b88",
          "url": "https://github.com/maidsafe/safe_network/commit/ddb939d5831b2f0d66fa2e0954b62e5e22a3ee69"
        },
        "date": 1652169795977,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 29403372641,
            "range": "± 11721495580",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3938469992,
            "range": "± 1112637309",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10025441626,
            "range": "± 18880013",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 10103620411,
            "range": "± 34491244",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 10448801455,
            "range": "± 97343424",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "375424bab5dca59adddcc6b691ba0deac09a1bcb",
          "message": "Merge #1168\n\n1168: Check incoming authed SAP in signed votes r=joshuef a=grumbach\n\n<!--\r\nThanks for contributing to the project! We recommend you check out our \"Guide to contributing\" page if you haven't already: https://github.com/maidsafe/QA/blob/master/CONTRIBUTING.md\r\n\r\nWrite your comment below this line: -->\r\n\r\n- Make sure incoming vote's auth'd SAPs are correctly signed by elders before we forward them to consensus. \n\nCo-authored-by: grumbach <anselmega@gmail.com>\nCo-authored-by: joshuef <joshuef@gmail.com>",
          "timestamp": "2022-05-10T13:18:44Z",
          "tree_id": "ba428bcde6d9df705a037903f0c52bb8f53b7c2b",
          "url": "https://github.com/maidsafe/safe_network/commit/375424bab5dca59adddcc6b691ba0deac09a1bcb"
        },
        "date": 1652194406996,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10590998110,
            "range": "± 11916595101",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3815738016,
            "range": "± 1013151763",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 10025800513,
            "range": "± 1808657133",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10039047148,
            "range": "± 4276469",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3251944284,
            "range": "± 1004834798",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4747493450,
            "range": "± 1018071236",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "0cd217240a72b462e04314bd67e1bbaf054374c0",
          "message": "Merge #1176\n\n1176: ci: use different mechanism for crate publishing r=joshuef a=jacderida\n\nThe last two release runs have failed when attempting to publish `sn_node`. It seems even if the\r\ndependent crate is returned in a `cargo search`, this doesn't ensure the crate will still be\r\navailable when you attempt to publish the crate with the dependency.\r\n\r\nNow the script is changed to simply just attempt the publish itself in a retry loop.\r\n\n\nCo-authored-by: Chris O'Neil <chriso83@protonmail.com>",
          "timestamp": "2022-05-10T14:27:35Z",
          "tree_id": "6e26b82d1742baaa4e823cd71666ac45dd94c825",
          "url": "https://github.com/maidsafe/safe_network/commit/0cd217240a72b462e04314bd67e1bbaf054374c0"
        },
        "date": 1652198444625,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10581848212,
            "range": "± 10875383194",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3789798537,
            "range": "± 744269048",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 10950498980,
            "range": "± 1520434049",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10037338064,
            "range": "± 7144857",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3185392519,
            "range": "± 1106009314",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4532772556,
            "range": "± 212531258",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "action@github.com",
            "name": "GitHub Action",
            "username": "actions-user"
          },
          "committer": {
            "email": "action@github.com",
            "name": "GitHub Action",
            "username": "actions-user"
          },
          "distinct": true,
          "id": "66638f508ad4df12b757672df589ba8ad09fbdfc",
          "message": "chore(release): sn_dysfunction-0.1.3/sn_node-0.58.17",
          "timestamp": "2022-05-11T05:54:18Z",
          "tree_id": "d2c4db6564dcc4d387dee9e6dfa37d926766f47d",
          "url": "https://github.com/maidsafe/safe_network/commit/66638f508ad4df12b757672df589ba8ad09fbdfc"
        },
        "date": 1652249942975,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10502418442,
            "range": "± 11455771948",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3783336950,
            "range": "± 73031950",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 9187762378,
            "range": "± 2105489318",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10034485522,
            "range": "± 7795307",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3253183910,
            "range": "± 1091504924",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 5031028020,
            "range": "± 2043173546",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "a03107ea7ea8a393c818a193eb2489e92cbbda20",
          "message": "Merge #1127\n\n1127: Back pressure debug r=joshuef a=joshuef\n\nPreviously we had backpressure running in response to every single message that came in. \r\n\r\nThis was somehow disrupting section formation and DKG. Moving it to an optional feature helped stabilise the base test cases in `main` w/ the membership changes.\r\n\r\nHere, I look to reenable backpressure as a default feature. \r\n\r\nI've moved it away from being triggered on _every_ incoming message, to be a periodic check on a loop over an interval.\r\n\r\nIt also now _only_ fires back pressure messages to the same section. \r\n\r\n(As things stand, intersection comms is currently limited to AE messages flows which should not overwhelm our nodes if backpressure within the section - which is where most messages arise - is working properly).\r\n\r\n(We _could_ fire backpressure to every connected node peer eg, but we don't know if they are clients so may be wasting time there...)\n\nCo-authored-by: Josh Wilson <joshuef@gmail.com>",
          "timestamp": "2022-05-11T07:49:06Z",
          "tree_id": "8e96b1cfcd3a7fb4fee48ef75be8ff2e9e007ece",
          "url": "https://github.com/maidsafe/safe_network/commit/a03107ea7ea8a393c818a193eb2489e92cbbda20"
        },
        "date": 1652261131786,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 11368344806,
            "range": "± 4775909654",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 5084993735,
            "range": "± 1240282821",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 11928699575,
            "range": "± 13817389324",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10054108776,
            "range": "± 2958516836",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3176033287,
            "range": "± 1117215845",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4724682921,
            "range": "± 126909029",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "action@github.com",
            "name": "GitHub Action",
            "username": "actions-user"
          },
          "committer": {
            "email": "action@github.com",
            "name": "GitHub Action",
            "username": "actions-user"
          },
          "distinct": true,
          "id": "a49a007ef8fde53a346403824f09eb0fd25e1109",
          "message": "chore(release): sn_interface-0.2.3/sn_node-0.58.18",
          "timestamp": "2022-05-12T05:58:05Z",
          "tree_id": "d458d2f0695b5b034cace43aa3733aead428186b",
          "url": "https://github.com/maidsafe/safe_network/commit/a49a007ef8fde53a346403824f09eb0fd25e1109"
        },
        "date": 1652336616468,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10483362673,
            "range": "± 11158565823",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 5156808636,
            "range": "± 1619629964",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 9253695671,
            "range": "± 13398857683",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10041012077,
            "range": "± 3782502",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3215999580,
            "range": "± 1026352337",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4596943556,
            "range": "± 217421356",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joshuef@gmail.com",
            "name": "Josh Wilson",
            "username": "joshuef"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "53ee4c51b82ebd0060c9adba32dac1a102890120",
          "message": "chore: simplify cleanupPeerLinks\n\nThere was a suspected deadlock in the CleanUpPeerLinks code, so here\nwe simplify things in order to hopefully prevent any deadlock.\n\nMoving the cleanup into comms, removing any checks against membership\n(as all nodes should be connectable; clients can always retry).\n\nAnd removing PeerLinks that are not conncted at all.",
          "timestamp": "2022-05-12T15:46:54+02:00",
          "tree_id": "a49b0fbb25d8f73eb52c08c24ceeac10d8843d20",
          "url": "https://github.com/maidsafe/safe_network/commit/53ee4c51b82ebd0060c9adba32dac1a102890120"
        },
        "date": 1652364968994,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10589199804,
            "range": "± 7102132699",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 4216381308,
            "range": "± 1642902972",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 11622225552,
            "range": "± 14091508127",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10041620698,
            "range": "± 1207624171",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3162304287,
            "range": "± 36878522",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4763683877,
            "range": "± 1369550940",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "aed6d5050c0b2cc37cc66d4c7b6ada70ee79808a",
          "message": "Merge #1180\n\n1180: feat: sort relocate candidates by distance to the churn_id r=maqi a=maqi\n\n<!--\r\nThanks for contributing to the project! We recommend you check out our \"Guide to contributing\" page if you haven't already: https://github.com/maidsafe/QA/blob/master/CONTRIBUTING.md\r\n\r\nWrite your comment below this line: -->\r\n\n\nCo-authored-by: qima <qi.ma@maidsafe.net>",
          "timestamp": "2022-05-12T14:21:48Z",
          "tree_id": "f1da979892003132abd65661146051209b1efc70",
          "url": "https://github.com/maidsafe/safe_network/commit/aed6d5050c0b2cc37cc66d4c7b6ada70ee79808a"
        },
        "date": 1652370492298,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10522568273,
            "range": "± 3671583072",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3898508367,
            "range": "± 1650421959",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 10351547389,
            "range": "± 1971303598",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10038842551,
            "range": "± 2215329891",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3141319223,
            "range": "± 26294674",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 5527784470,
            "range": "± 1367533875",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "2062bcee463c46f028991374d1d548b848f9052b",
          "message": "Merge #1181\n\n1181: fix: publishing script not exiting correctly r=joshuef a=jacderida\n\nIn the last release run, publishing of crates were successful with the retry loop; however, the\r\nscript didn't exit correctly on the successful publish. The loop then continued until the retries\r\nwere exceeded.\r\n\r\nThe problem was the use of the sub shell. I use this from habit. Whenever a script is changing\r\ndirectories, using a sub shell means the change only applies inside the sub shell, so the outer\r\nscript will retain its current directory, which is often desirable behaviour. The use of `exit 0` inside\r\nthe sub shell only exits the sub shell, not the entire script, as was intended.\r\n\r\nI've now removed the use of the sub shell; it wasn't really a necessary protection for this scenario\r\nanyway.\n\nCo-authored-by: Chris O'Neil <chriso83@protonmail.com>",
          "timestamp": "2022-05-12T16:23:44Z",
          "tree_id": "517d2b8150664895dd39baaf44315ceae451a025",
          "url": "https://github.com/maidsafe/safe_network/commit/2062bcee463c46f028991374d1d548b848f9052b"
        },
        "date": 1652378014007,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10540740553,
            "range": "± 5764774328",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3942365799,
            "range": "± 969110640",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 10246829357,
            "range": "± 580875217",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10044084948,
            "range": "± 2947287946",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3144270637,
            "range": "± 20372175",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4791109934,
            "range": "± 178101852",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "action@github.com",
            "name": "GitHub Action",
            "username": "actions-user"
          },
          "committer": {
            "email": "action@github.com",
            "name": "GitHub Action",
            "username": "actions-user"
          },
          "distinct": true,
          "id": "aeb2945e164ca9a07390b4b7fc5220daf07f9401",
          "message": "chore(release): sn_node-0.58.19",
          "timestamp": "2022-05-13T05:50:40Z",
          "tree_id": "ed583227244ac52b33a76f5f5f12542d5218817e",
          "url": "https://github.com/maidsafe/safe_network/commit/aeb2945e164ca9a07390b4b7fc5220daf07f9401"
        },
        "date": 1652422581821,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10610288375,
            "range": "± 13713516381",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 4005305254,
            "range": "± 1033692588",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 9890122204,
            "range": "± 1671243206",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10036642100,
            "range": "± 3617749405",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3280311253,
            "range": "± 1211807993",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4827116265,
            "range": "± 844871233",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "b5a222c7facb6f1617281ed1133464a435db01f8",
          "message": "Merge #1182\n\n1182: chore(node): dont hold comms session lock over session cleanup r=Yoga07 a=joshuef\n\nIt's still not clear if this is where we may be deadlocking. But moving to hold the lock over a shorter duration certainly seems sensible\r\n\r\n<!--\r\nThanks for contributing to the project! We recommend you check out our \"Guide to contributing\" page if you haven't already: https://github.com/maidsafe/QA/blob/master/CONTRIBUTING.md\r\n\r\nWrite your comment below this line: -->\r\n\n\nCo-authored-by: Josh Wilson <joshuef@gmail.com>",
          "timestamp": "2022-05-16T07:09:04Z",
          "tree_id": "f165d5a2f5d01d7a99e202ee9f7a184d8edd41ec",
          "url": "https://github.com/maidsafe/safe_network/commit/b5a222c7facb6f1617281ed1133464a435db01f8"
        },
        "date": 1652690840889,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10899572100,
            "range": "± 11149293090",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3836230820,
            "range": "± 1176872445",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 9166083337,
            "range": "± 1626066669",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10038979674,
            "range": "± 5474574",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 4317839131,
            "range": "± 1223529412",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4549476477,
            "range": "± 1371209155",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "f22b19dc1e391cc5f5409f4cec2d664ad199cbcc",
          "message": "Merge #1184\n\n1184: Tidy cmd insp r=joshuef a=joshuef\n\n<!--\r\nThanks for contributing to the project! We recommend you check out our \"Guide to contributing\" page if you haven't already: https://github.com/maidsafe/QA/blob/master/CONTRIBUTING.md\r\n\r\nWrite your comment below this line: -->\r\n\n\nCo-authored-by: Josh Wilson <joshuef@gmail.com>",
          "timestamp": "2022-05-16T11:06:49Z",
          "tree_id": "b942effdfd37d43afa2b6338bfa6cb49fe95acf7",
          "url": "https://github.com/maidsafe/safe_network/commit/f22b19dc1e391cc5f5409f4cec2d664ad199cbcc"
        },
        "date": 1652704761919,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10524442085,
            "range": "± 9469697674",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3957116342,
            "range": "± 986377851",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 9675771949,
            "range": "± 976804204",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10045069890,
            "range": "± 8599462",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3167756367,
            "range": "± 729300553",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4778565811,
            "range": "± 1578383447",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "action@github.com",
            "name": "GitHub Action",
            "username": "actions-user"
          },
          "committer": {
            "email": "action@github.com",
            "name": "GitHub Action",
            "username": "actions-user"
          },
          "distinct": true,
          "id": "332e8f126e0e9351e8698ce2604e6fdd8ce6f7b5",
          "message": "chore(release): sn_node-0.58.20",
          "timestamp": "2022-05-17T05:49:19Z",
          "tree_id": "1782f1a96a54ac7947d4281410e2f61a9af806ab",
          "url": "https://github.com/maidsafe/safe_network/commit/332e8f126e0e9351e8698ce2604e6fdd8ce6f7b5"
        },
        "date": 1652768014148,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10586829824,
            "range": "± 8124976236",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 4183122686,
            "range": "± 4116474826",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 9658746014,
            "range": "± 2968422210",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10035392534,
            "range": "± 2949312446",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3141769110,
            "range": "± 22730585",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 5408960638,
            "range": "± 2256212214",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "afda86c5bd759f6a19cb921c356fad51f76daecd",
          "message": "Merge #1150\n\n1150: Handover w/ Dkg Generations r=Yoga07 a=joshuef\n\n<!--\r\nThanks for contributing to the project! We recommend you check out our \"Guide to contributing\" page if you haven't already: https://github.com/maidsafe/QA/blob/master/CONTRIBUTING.md\r\n\r\nWrite your comment below this line: -->\r\n\n\nCo-authored-by: Josh Wilson <joshuef@gmail.com>\nCo-authored-by: joshuef <joshuef@gmail.com>",
          "timestamp": "2022-05-17T08:48:13Z",
          "tree_id": "3a16762159b3a6b44a1209349dfd560c47bddb01",
          "url": "https://github.com/maidsafe/safe_network/commit/afda86c5bd759f6a19cb921c356fad51f76daecd"
        },
        "date": 1652782705805,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10675531882,
            "range": "± 10774757460",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 4971001748,
            "range": "± 5225123709",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 9368380750,
            "range": "± 352857936",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10028947787,
            "range": "± 3616264186",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3262809019,
            "range": "± 709980154",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4605986786,
            "range": "± 973288876",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "00f41b4a96bcc172d91620aa0da0cb799db5483c",
          "message": "Merge #1189\n\n1189: chore(client): send some msgs in bg threads r=Yoga07 a=joshuef\n\nThis should unblock client threads on initial contact and on queries\r\n\n\nCo-authored-by: Josh Wilson <joshuef@gmail.com>",
          "timestamp": "2022-05-17T10:12:40Z",
          "tree_id": "9ea566556ffd9e75b38b76ac476b7b23a9802042",
          "url": "https://github.com/maidsafe/safe_network/commit/00f41b4a96bcc172d91620aa0da0cb799db5483c"
        },
        "date": 1652787734725,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10678747747,
            "range": "± 10965837275",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3931954961,
            "range": "± 1272012386",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 11690036637,
            "range": "± 2457184937",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10041521243,
            "range": "± 2957920946",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3190265082,
            "range": "± 1108116517",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4861978381,
            "range": "± 1016525596",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "8833cb8a4ae13f04ea86c67e92fce4d82a107f5a",
          "message": "Merge #1190\n\n1190: chore(deps): upgrade blsttc to v5.2.0 and rand to v0.8 r=bochaco a=bochaco\n\n\n\nCo-authored-by: bochaco <gabrielviganotti@gmail.com>",
          "timestamp": "2022-05-17T20:48:05Z",
          "tree_id": "f104c83a664ca6af1445ed66ae1576e1e676b356",
          "url": "https://github.com/maidsafe/safe_network/commit/8833cb8a4ae13f04ea86c67e92fce4d82a107f5a"
        },
        "date": 1652826342900,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10897732835,
            "range": "± 9252957262",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3703284580,
            "range": "± 4315651296",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 9496399165,
            "range": "± 901457698",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10031778894,
            "range": "± 4895385",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3118777706,
            "range": "± 26525409",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4465393287,
            "range": "± 987191974",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "action@github.com",
            "name": "GitHub Action",
            "username": "actions-user"
          },
          "committer": {
            "email": "action@github.com",
            "name": "GitHub Action",
            "username": "actions-user"
          },
          "distinct": true,
          "id": "9b06304f46e1a1bda90a0fc6ff82edc928c2529d",
          "message": "chore(release): sn_interface-0.2.4/sn_client-0.62.3/sn_api-0.60.2/sn_cli-0.53.1",
          "timestamp": "2022-05-18T05:57:40Z",
          "tree_id": "716a9ed6d3c5170fcb640721e3eb3bd1e02ea8e2",
          "url": "https://github.com/maidsafe/safe_network/commit/9b06304f46e1a1bda90a0fc6ff82edc928c2529d"
        },
        "date": 1652855092176,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10792598788,
            "range": "± 10801548174",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3804107556,
            "range": "± 1631302096",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 8928028969,
            "range": "± 7841967750",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10029914332,
            "range": "± 2954878535",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 4297358122,
            "range": "± 1224328465",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4389428856,
            "range": "± 1074571286",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joshuef@gmail.com",
            "name": "Josh Wilson",
            "username": "joshuef"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "06591a11458adb5cfd917cc1239371acf4f8834f",
          "message": "fix: prevent deadlock in lru cache impl.\n\nWe were locking over the queue, and then attempting to purge the queue\nwithin the self.priority() func, which required a lock",
          "timestamp": "2022-05-18T14:41:34+02:00",
          "tree_id": "f96fe5585ac2379b11ee240ce01a2d0655094577",
          "url": "https://github.com/maidsafe/safe_network/commit/06591a11458adb5cfd917cc1239371acf4f8834f"
        },
        "date": 1652880138087,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10657978365,
            "range": "± 12436176602",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3791136624,
            "range": "± 1005970087",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 9892655733,
            "range": "± 1724234126",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10030319879,
            "range": "± 2954835444",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3151077295,
            "range": "± 1617126179",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4556766449,
            "range": "± 253359281",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10676947399,
            "range": "± 11511627129",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3850042770,
            "range": "± 1803279632",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 8612633025,
            "range": "± 292375492",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10031653857,
            "range": "± 2215643636",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3143662011,
            "range": "± 989120942",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4741931028,
            "range": "± 1061306607",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "c5b0f1b6d4f288737bc1f4fbda162386149ec402",
          "message": "Merge #1193\n\n1193: Fix dysf potential deadlock r=joshuef a=joshuef\n\n<!--\r\nThanks for contributing to the project! We recommend you check out our \"Guide to contributing\" page if you haven't already: https://github.com/maidsafe/QA/blob/master/CONTRIBUTING.md\r\n\r\nWrite your comment below this line: -->\r\n\n\nCo-authored-by: Josh Wilson <joshuef@gmail.com>",
          "timestamp": "2022-05-19T10:13:04Z",
          "tree_id": "f35ab4510abb063268e96693beaa67e866357691",
          "url": "https://github.com/maidsafe/safe_network/commit/c5b0f1b6d4f288737bc1f4fbda162386149ec402"
        },
        "date": 1652962147202,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 29902633261,
            "range": "± 11378651287",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3895061054,
            "range": "± 992064544",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 11976771758,
            "range": "± 2062978543",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10030633397,
            "range": "± 5196840",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3128892860,
            "range": "± 26030882",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 5051451237,
            "range": "± 894920695",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/single registers",
            "value": 8930,
            "range": "± 288",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/single chunks",
            "value": 136691,
            "range": "± 882",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "d9ba264a6b2b657dce60b5ded78f1cecd840dbb1",
          "message": "Merge #1178\n\n1178: Section probing for all nodes to trigger rejoin if needed r=davidrusu a=grumbach\n\n<!--\r\nThanks for contributing to the project! We recommend you check out our \"Guide to contributing\" page if you haven't already: https://github.com/maidsafe/QA/blob/master/CONTRIBUTING.md\r\n\r\nWrite your comment below this line: -->\r\n\r\n- Section probing for all nodes every now and then\r\n- Removed unused/ignored `dst` field in `AntiEntropyProbe`\r\n- Trigger `ChurnJoinMiss` error and event when nodes realise they are not members of a section they should be in during AE\r\n- Node rejoins/restart when they receive `ChurnJoinMiss`\n\nCo-authored-by: grumbach <anselmega@gmail.com>",
          "timestamp": "2022-05-19T21:46:30Z",
          "tree_id": "b4e6f9ae67ea53761957b1a434af1539ddc8872c",
          "url": "https://github.com/maidsafe/safe_network/commit/d9ba264a6b2b657dce60b5ded78f1cecd840dbb1"
        },
        "date": 1653002869067,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10789665333,
            "range": "± 11260843886",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3825045659,
            "range": "± 1110895824",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 9361545862,
            "range": "± 896846494",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10032534257,
            "range": "± 6499338",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3315862178,
            "range": "± 2198526145",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4534453446,
            "range": "± 1307539741",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/single registers",
            "value": 13467,
            "range": "± 398",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/single chunks",
            "value": 197771,
            "range": "± 4345",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "1d1689f91d0bc450257d1a279561ea7b0c1b71a7",
          "message": "Merge #1196\n\n1196: chore(messaging): add Display for OperationId r=joshuef a=joshuef\n\nBREAKING CHANGE: changes messsaging and OperationId types to get nicer\r\nOpId logging\r\n\r\n<!--\r\nThanks for contributing to the project! We recommend you check out our \"Guide to contributing\" page if you haven't already: https://github.com/maidsafe/QA/blob/master/CONTRIBUTING.md\r\n\r\nWrite your comment below this line: -->\r\n\n\nCo-authored-by: Josh Wilson <joshuef@gmail.com>",
          "timestamp": "2022-05-20T10:23:20Z",
          "tree_id": "b0eb95d481e3f8d05f6f1ac64b64592712f35980",
          "url": "https://github.com/maidsafe/safe_network/commit/1d1689f91d0bc450257d1a279561ea7b0c1b71a7"
        },
        "date": 1653047489479,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10493873529,
            "range": "± 6868597216",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3759876511,
            "range": "± 1021707101",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 9625789987,
            "range": "± 1063576212",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10027740842,
            "range": "± 1633421037",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3193285995,
            "range": "± 1120211410",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4522983135,
            "range": "± 300357690",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/single registers",
            "value": 13451,
            "range": "± 586",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/single chunks",
            "value": 197984,
            "range": "± 2452",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "action@github.com",
            "name": "GitHub Action",
            "username": "actions-user"
          },
          "committer": {
            "email": "action@github.com",
            "name": "GitHub Action",
            "username": "actions-user"
          },
          "distinct": true,
          "id": "cf21d66b9b726123e0a4320cd68481b67f7af03d",
          "message": "chore(release): sn_dysfunction-0.2.0/sn_client-0.63.0/sn_node-0.59.0/sn_api-0.61.0/sn_cli-0.54.0",
          "timestamp": "2022-05-21T18:23:11Z",
          "tree_id": "adb1dac9a34e0fb128e3e615adbe3c136ec444cb",
          "url": "https://github.com/maidsafe/safe_network/commit/cf21d66b9b726123e0a4320cd68481b67f7af03d"
        },
        "date": 1653159771225,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 20024737079,
            "range": "± 12057501578",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3733389696,
            "range": "± 1034644847",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 9849106782,
            "range": "± 2142296948",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10031078452,
            "range": "± 2956315040",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3140173639,
            "range": "± 726361702",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4549389044,
            "range": "± 1051471244",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/single registers",
            "value": 13947,
            "range": "± 385",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/single chunks",
            "value": 205801,
            "range": "± 1121",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "chriso83@protonmail.com",
            "name": "Chris O'Neil",
            "username": "jacderida"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "bd24e72036c602f3978b8cf4a742cd293dcb2100",
          "message": "ci: additional check in version bumping process\n\nPreviously we were checking for a particular piece of text in the smart-release output, but it turns\nout there can also be other text that indicates that a crate will have changes. Not sure how it was\nmissed until this point.",
          "timestamp": "2022-05-21T21:30:59+02:00",
          "tree_id": "5c1266fb1ae5ef708637a3e29c956ec65a55f4b4",
          "url": "https://github.com/maidsafe/safe_network/commit/bd24e72036c602f3978b8cf4a742cd293dcb2100"
        },
        "date": 1653163241067,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10894240148,
            "range": "± 13854775972",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3921021816,
            "range": "± 4660443949",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 9809693450,
            "range": "± 1213602960",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10032642378,
            "range": "± 2952023335",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3188139696,
            "range": "± 961139533",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4781621412,
            "range": "± 951689882",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/single registers",
            "value": 15497,
            "range": "± 662",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/single chunks",
            "value": 243054,
            "range": "± 6240",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "c6e6e324164028c6c15a78643783a9f86679f39e",
          "message": "Merge #1195\n\n1195: refactor(node_state): move NodeState validations to NodeState struct r=davidrusu a=davidrusu\n\nThis is part of the work to move membership history to NetworkKnowledge. By moving NodeState validations to the NodeState struct, we have less code in Membership and so, less code to move to NetworkKnowledge.\n\nCo-authored-by: David Rusu <davidrusu.me@gmail.com>\nCo-authored-by: David Irvine <david.irvine@maidsafe.net>",
          "timestamp": "2022-05-24T17:50:40Z",
          "tree_id": "6f89bda404265b1affb34f7f56d7a37ab47bc239",
          "url": "https://github.com/maidsafe/safe_network/commit/c6e6e324164028c6c15a78643783a9f86679f39e"
        },
        "date": 1653420686654,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10787247250,
            "range": "± 11870124342",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3695483661,
            "range": "± 1038236290",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 8650492152,
            "range": "± 1267229647",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10028078392,
            "range": "± 2216826133",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 4298178808,
            "range": "± 1615015716",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4424916862,
            "range": "± 962399949",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/single registers",
            "value": 12214,
            "range": "± 412",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/single chunks",
            "value": 201157,
            "range": "± 1863",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "action@github.com",
            "name": "GitHub Action",
            "username": "actions-user"
          },
          "committer": {
            "email": "action@github.com",
            "name": "GitHub Action",
            "username": "actions-user"
          },
          "distinct": true,
          "id": "ef56cf9cf8de45a9f13c2510c63de245b12aeae8",
          "message": "chore(release): sn_interface-0.4.0/sn_dysfunction-0.3.0/sn_client-0.64.0/sn_node-0.60.0/sn_api-0.62.0/sn_cli-0.55.0",
          "timestamp": "2022-05-25T06:14:08Z",
          "tree_id": "3fae60e36c7cbd54379a55a39de40dee333e19ff",
          "url": "https://github.com/maidsafe/safe_network/commit/ef56cf9cf8de45a9f13c2510c63de245b12aeae8"
        },
        "date": 1653461070059,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10667526006,
            "range": "± 14216182437",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3763599098,
            "range": "± 1019121698",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 11158539092,
            "range": "± 2086102407",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10030384651,
            "range": "± 5538850",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3146750137,
            "range": "± 741077429",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4581130259,
            "range": "± 873973582",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/single registers",
            "value": 13537,
            "range": "± 426",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/single chunks",
            "value": 204859,
            "range": "± 2217",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "cd32ca6535b17aedacfb4051e97e4b3540bb8a71",
          "message": "Merge #1203\n\n1203: chore(deps): bump consensus 1.16.0 -> 2.0.0 r=joshuef a=davidrusu\n\n\n\nCo-authored-by: David Rusu <davidrusu.me@gmail.com>\nCo-authored-by: joshuef <joshuef@gmail.com>",
          "timestamp": "2022-05-25T07:15:17Z",
          "tree_id": "8a0033874e8a4109fbe63a055fa73a809ed1c63d",
          "url": "https://github.com/maidsafe/safe_network/commit/cd32ca6535b17aedacfb4051e97e4b3540bb8a71"
        },
        "date": 1653468712342,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 20049256293,
            "range": "± 10246841038",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3721932456,
            "range": "± 755811708",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 8629974350,
            "range": "± 7000810009",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10031656430,
            "range": "± 2215167352",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3307264945,
            "range": "± 1170466748",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4348584942,
            "range": "± 780068251",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/single registers",
            "value": 13267,
            "range": "± 402",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/single chunks",
            "value": 189966,
            "range": "± 10871",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "5e82ef3d0e78898f9ffac8bebe4970c4d26e608f",
          "message": "Merge #1198 #1204\n\n1198: chore: remove test-utils references from readme r=joshuef a=RolandSherwin\n\n\n\n1204: fix: publish sn_interface befor dysfunction now we depend on sn_int t… r=Yoga07 a=joshuef\n\n…here\r\n\r\n<!--\r\nThanks for contributing to the project! We recommend you check out our \"Guide to contributing\" page if you haven't already: https://github.com/maidsafe/QA/blob/master/CONTRIBUTING.md\r\n\r\nWrite your comment below this line: -->\r\n\n\nCo-authored-by: RolandSherwin <RolandSherwin@protonmail.com>\nCo-authored-by: Josh Wilson <joshuef@gmail.com>",
          "timestamp": "2022-05-25T08:22:15Z",
          "tree_id": "320ed5714f1c050f5a00dde3768191ce8673a78e",
          "url": "https://github.com/maidsafe/safe_network/commit/5e82ef3d0e78898f9ffac8bebe4970c4d26e608f"
        },
        "date": 1653473154171,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10892536554,
            "range": "± 10783985827",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3963227835,
            "range": "± 1127043171",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 11366797453,
            "range": "± 13853128637",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10031545703,
            "range": "± 3382855681",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3174224570,
            "range": "± 1154801652",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 5147968447,
            "range": "± 2663147537",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/single registers",
            "value": 17444,
            "range": "± 819",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/single chunks",
            "value": 284922,
            "range": "± 11611",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "6c9b851dd5bab8b2f5d9b3ef1db72d198706ac9d",
          "message": "Merge #1208\n\n1208: Feat constant replication flow r=joshuef a=joshuef\n\nTo give an idea, adding:\r\n\r\nSendingDataReplicationBatch logs to [here](https://github.com/maidsafe/safe_network/blob/main/sn_node/src/node/api/dispatcher.rs#L563-L567), on main, we see over the course of the network split test, updated w/ same values as churn in this PR\r\n\r\n- it's `found: 312 times`. Each one of those is ~50mb. a total of ~15.6gb\r\n- memory jumps high. I have to kill it when it reaches ~700mb per node.\r\n\r\nWith this PR, over the course of the `churn` test:\r\n\r\n- nodes dont go over ~110mb\r\n- it succeeds\r\n- SendingMissingReplicatedData found: 3287 times, 3.2gb... \r\n\r\nThis is a slower rate of transfer, which is safer for nodes/systems running nodes, that can more easily be tweaked by the node themselves down the line. (And add in there that there should be less mem per message too!)\r\n\n\nCo-authored-by: Josh Wilson <joshuef@gmail.com>",
          "timestamp": "2022-05-25T15:41:23Z",
          "tree_id": "66be106e42c5fc341bd10d23894a65887f4c79df",
          "url": "https://github.com/maidsafe/safe_network/commit/6c9b851dd5bab8b2f5d9b3ef1db72d198706ac9d"
        },
        "date": 1653499115594,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10599642286,
            "range": "± 12166461744",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3946638237,
            "range": "± 4881230378",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 9426413778,
            "range": "± 2588154174",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 8870437115,
            "range": "± 1633221793",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3147783031,
            "range": "± 45550812",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4597580387,
            "range": "± 992719351",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/single registers",
            "value": 16307,
            "range": "± 1493",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/single chunks",
            "value": 266576,
            "range": "± 14439",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "e42a2e3c212597e68238451a5bb4a8725c4761be",
          "message": "Merge #1202\n\n1202: Allow handover re-consensus when elders agree on an empty set r=davidrusu a=grumbach\n\n<!--\r\nThanks for contributing to the project! We recommend you check out our \"Guide to contributing\" page if you haven't already: https://github.com/maidsafe/QA/blob/master/CONTRIBUTING.md\r\n\r\nWrite your comment below this line: -->\r\n\r\nFixes  #1186\r\n\r\n- generations for handover\r\n- AE for handover \r\n- empty consensus handling\r\n\n\nCo-authored-by: grumbach <anselmega@gmail.com>",
          "timestamp": "2022-05-25T16:54:04Z",
          "tree_id": "8f2e6d2eb36ed053bd433d8a8ab02bbcb44127a0",
          "url": "https://github.com/maidsafe/safe_network/commit/e42a2e3c212597e68238451a5bb4a8725c4761be"
        },
        "date": 1653503589704,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10674878843,
            "range": "± 13638485774",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3802507267,
            "range": "± 3392952056",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 8725453105,
            "range": "± 130857901",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10030943979,
            "range": "± 2216036232",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3193165510,
            "range": "± 1196465545",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4566378665,
            "range": "± 1942694836",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/single registers",
            "value": 13907,
            "range": "± 480",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/single chunks",
            "value": 210202,
            "range": "± 2943",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "c08fbb94e9306a00cdd24db9be73f903cb1f3362",
          "message": "Merge #1210\n\n1210: chore: explicitly drop Node on join retry r=joshuef a=joshuef\n\nAdd more logs to aid debugging\r\n\r\n<!--\r\nThanks for contributing to the project! We recommend you check out our \"Guide to contributing\" page if you haven't already: https://github.com/maidsafe/QA/blob/master/CONTRIBUTING.md\r\n\r\nWrite your comment below this line: -->\r\n\n\nCo-authored-by: Josh Wilson <joshuef@gmail.com>",
          "timestamp": "2022-05-26T11:50:57Z",
          "tree_id": "990ca83f0faf67ceaef31622d2dc09d1714a3db8",
          "url": "https://github.com/maidsafe/safe_network/commit/c08fbb94e9306a00cdd24db9be73f903cb1f3362"
        },
        "date": 1653572129359,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 16934387257,
            "range": "± 6853278743",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3733997664,
            "range": "± 79586206",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 8624279748,
            "range": "± 7076849952",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10032349656,
            "range": "± 5964826",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3138284088,
            "range": "± 960065581",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4412817351,
            "range": "± 1585598129",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/single registers",
            "value": 13152,
            "range": "± 323",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/single chunks",
            "value": 202708,
            "range": "± 5925",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joshuef@gmail.com",
            "name": "Josh Wilson",
            "username": "joshuef"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "77d962abb97f8b00e9295419079b43224ca67341",
          "message": "fix: shutdown runtime in node loop\n\nthis should hopefully shutdown any bg processes running which may be blocking ports on reconnect\n\nCo-authored-by: David Rusu <david.rusu@maidsafe.net>",
          "timestamp": "2022-05-26T18:19:41+02:00",
          "tree_id": "22511c2f72965aa85c22e9be55e4f94a3aafc5cc",
          "url": "https://github.com/maidsafe/safe_network/commit/77d962abb97f8b00e9295419079b43224ca67341"
        },
        "date": 1653583591456,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10494057124,
            "range": "± 11950525715",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3762241576,
            "range": "± 1014426604",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 8478328061,
            "range": "± 274936849",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10031061976,
            "range": "± 4102343",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3143799096,
            "range": "± 1119518312",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4580146631,
            "range": "± 899492640",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/single registers",
            "value": 13740,
            "range": "± 745",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/single chunks",
            "value": 200696,
            "range": "± 1649",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "action@github.com",
            "name": "GitHub Action",
            "username": "actions-user"
          },
          "committer": {
            "email": "action@github.com",
            "name": "GitHub Action",
            "username": "actions-user"
          },
          "distinct": true,
          "id": "e5fcd032e1dd904e05bc23e119af1d06e3b85a06",
          "message": "chore(release): sn_interface-0.5.0/sn_dysfunction-0.4.0/sn_client-0.65.0/sn_node-0.61.0/sn_api-0.63.0/sn_cli-0.56.0",
          "timestamp": "2022-05-27T06:35:17Z",
          "tree_id": "d4c2df0a126f8fdda4202ff17fb3e176aaa6feb8",
          "url": "https://github.com/maidsafe/safe_network/commit/e5fcd032e1dd904e05bc23e119af1d06e3b85a06"
        },
        "date": 1653635098740,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10682839559,
            "range": "± 11181164137",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3690206161,
            "range": "± 2021774330",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 8503365372,
            "range": "± 9215649300",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10031458252,
            "range": "± 6168918",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3160969236,
            "range": "± 956473067",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4574817021,
            "range": "± 1273743473",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/single registers",
            "value": 11899,
            "range": "± 439",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/single chunks",
            "value": 203869,
            "range": "± 1671",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "f9fc2a76f083ba5161c8c4eef9013c53586b4693",
          "message": "Merge #1192\n\n1192: Chore improve data storage bench r=Yoga07 a=joshuef\n\n<!--\r\nThanks for contributing to the project! We recommend you check out our \"Guide to contributing\" page if you haven't already: https://github.com/maidsafe/QA/blob/master/CONTRIBUTING.md\r\n\r\nWrite your comment below this line: -->\r\n\n\nCo-authored-by: Josh Wilson <joshuef@gmail.com>\nCo-authored-by: joshuef <joshuef@gmail.com>",
          "timestamp": "2022-05-31T10:31:42Z",
          "tree_id": "9a60ea000ad5e3ba03363a15141bef9f31154753",
          "url": "https://github.com/maidsafe/safe_network/commit/f9fc2a76f083ba5161c8c4eef9013c53586b4693"
        },
        "date": 1654000035076,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10484483751,
            "range": "± 10793071694",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3901086360,
            "range": "± 1202820560",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 9351184399,
            "range": "± 6699222484",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10038758543,
            "range": "± 3621762534",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3125358604,
            "range": "± 2192258942",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4675089130,
            "range": "± 990621377",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "e53f832b677b0489d1a578f8ee6564f054c6c288",
          "message": "Merge #1218\n\n1218: Reissue DBC to a Particular Owner r=davidrusu a=jacderida\n\n- cd85844f9 **feat!: reissue dbc to a particular owner**\r\n\r\n  BREAKING CHANGE: the `wallet_reissue` API is extended to add an `owner_public_key` argument. This is\r\n  an `Option<bls::PublicKey>`. To reissue to a bearer DBC, use `None`, but to reissue to a particular\r\n  owner, specify the key.\r\n\r\n  I've also taken an opportunity to update the API documentation a little. In the doc comments and\r\n  logging output, I changed \"Wallet\" references to \"wallet\": in this context, \"wallet\" is not a proper\r\n  noun, so it shouldn't be capitalised (unless it happens to be at the start of a sentence). If you\r\n  wanted to refer to the type `Wallet`, that would be capitalised, but it should also be referred to\r\n  using enclosing backticks.\r\n\r\n- a1c6aebe6 **feat!: wallet deposit read input dbc from stdin**\r\n\r\n  BREAKING CHANGE: the positional `dbc` argument is changed to an optional argument.\r\n\r\n  The DBC hex representation is 8000+ characters long, so this gives the user the option to provide\r\n  the large input DBC from a file, like so:\r\n  ```\r\n  safe wallet deposit <safe-url> < ~/.safe/input_dbc\r\n  ```\r\n\r\n  Bash seems to allow you to provide this huge argument on the command line, but it doesn't work in\r\n  Fish, which is the shell I use.\r\n\r\n- e548388c6 **chore: upgrade sn_dbc to 3.2.0**\r\n\r\n  This new release has utilities for serializing/deserializing `Dbc` to/from hex.\r\n\r\n- 0d97494f7 **feat: add public key argument for owned dbcs**\r\n\r\n  The `wallet reissue` command now has an additional optional argument, `--public-key`, which allows\r\n  the user to reissue a DBC to be owned by the holder of that public key. The key should be BLS\r\n  hex-encoded.\r\n\r\n  The `wallet deposit` command will now require extension to provide the secret key when depositing an\r\n  owned DBC. This will be done as a separate piece of work.\r\n\r\n  Some additional changes were made in support or to tidy CLI-related code:\r\n  * The conversion of DBCs to/from hex were removed from the CLI since this is now done on the `Dbc`\r\n    type.\r\n  * A CLI test that existed to test the above conversion code was removed since it's no longer\r\n    necessary.\r\n  * The naming scheme for the CLI wallet tests were elaborated and the redundant \"calling_safe\"\r\n    prefixes were removed.\r\n\n\nCo-authored-by: Chris O'Neil <chriso83@protonmail.com>",
          "timestamp": "2022-06-03T20:02:54Z",
          "tree_id": "1febecc7f1a4becd9495ec5b28e8f1af9fd043d3",
          "url": "https://github.com/maidsafe/safe_network/commit/e53f832b677b0489d1a578f8ee6564f054c6c288"
        },
        "date": 1654292100678,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10668875103,
            "range": "± 8597810207",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3867483170,
            "range": "± 386182566",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 9456625787,
            "range": "± 15720540333",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10032997509,
            "range": "± 5846599",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3174696058,
            "range": "± 1126790722",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4794462788,
            "range": "± 963112963",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "2f2604325d533357bad7d917315cf4cba0b2d3c0",
          "message": "Merge #1217\n\n1217: feat: handover sap elder checks with membership knowledge r=joshuef a=grumbach\n\n<!--\r\nThanks for contributing to the project! We recommend you check out our \"Guide to contributing\" page if you haven't already: https://github.com/maidsafe/QA/blob/master/CONTRIBUTING.md\r\n\r\nWrite your comment below this line: -->\r\n\r\nSquashed version of PR #1215  \r\nhttps://github.com/maidsafe/safe_network/pull/1215 \n\nCo-authored-by: grumbach <anselmega@gmail.com>\nCo-authored-by: Anselme <anselmega@gmail.com>",
          "timestamp": "2022-06-06T05:17:59Z",
          "tree_id": "94a735f10839b61bd05a6fde406559d445f5c482",
          "url": "https://github.com/maidsafe/safe_network/commit/2f2604325d533357bad7d917315cf4cba0b2d3c0"
        },
        "date": 1654498579287,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 19920333426,
            "range": "± 12015110995",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 6175017996,
            "range": "± 4059856155",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 10012005335,
            "range": "± 423994206",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10040388388,
            "range": "± 2218480054",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3165311613,
            "range": "± 1549762671",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4888834415,
            "range": "± 1309212446",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "992c4951670afc769feea7e6cd38db021aed88a7",
          "message": "Merge #1214\n\n1214: Gabriel spentbook PR1143 r=maqi a=maqi\n\n<!--\r\nThanks for contributing to the project! We recommend you check out our \"Guide to contributing\" page if you haven't already: https://github.com/maidsafe/QA/blob/master/CONTRIBUTING.md\r\n\r\nWrite your comment below this line: -->\r\n\r\nThis PR is mainly based on the PR 1143 to complete the rebase and conflicts resolving work to get merged for the initial work.\r\nThere is a list of detailed future work in the PR comment of PR 1143, which worth to be noted.\r\n\r\nCloses https://github.com/maidsafe/safe_network/pull/1143\r\n\n\nCo-authored-by: bochaco <gabrielviganotti@gmail.com>\nCo-authored-by: qima <qi.ma@maidsafe.net>\nCo-authored-by: joshuef <joshuef@gmail.com>",
          "timestamp": "2022-06-06T16:15:41Z",
          "tree_id": "4cc9f1df90890dae168866b614fef6710234d69d",
          "url": "https://github.com/maidsafe/safe_network/commit/992c4951670afc769feea7e6cd38db021aed88a7"
        },
        "date": 1654538313832,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 29498467146,
            "range": "± 11750789104",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3898408195,
            "range": "± 4305207951",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 9819099031,
            "range": "± 1922748649",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 8870901065,
            "range": "± 1632426334",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3159080018,
            "range": "± 24270211",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 6360998459,
            "range": "± 1633884850",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "action@github.com",
            "name": "GitHub Action",
            "username": "actions-user"
          },
          "committer": {
            "email": "action@github.com",
            "name": "GitHub Action",
            "username": "actions-user"
          },
          "distinct": true,
          "id": "9aa666763c381ed589343e306c583f558d935251",
          "message": "chore(release): sn_cli-0.57.1",
          "timestamp": "2022-06-08T05:57:57Z",
          "tree_id": "478d237428562ddbed91c98184d0bcc899682db9",
          "url": "https://github.com/maidsafe/safe_network/commit/9aa666763c381ed589343e306c583f558d935251"
        },
        "date": 1654669800156,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 20023406691,
            "range": "± 12194528404",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3689802296,
            "range": "± 1278880888",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 8249081102,
            "range": "± 1129566888",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10029814958,
            "range": "± 5712570",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3129477015,
            "range": "± 1133365671",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4528781552,
            "range": "± 2661668375",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "action@github.com",
            "name": "GitHub Action",
            "username": "actions-user"
          },
          "committer": {
            "email": "action@github.com",
            "name": "GitHub Action",
            "username": "actions-user"
          },
          "distinct": true,
          "id": "d05e7b3a97db73cdf84f74560056abe1f087820a",
          "message": "chore(release): sn_cli-0.57.2",
          "timestamp": "2022-06-09T06:01:57Z",
          "tree_id": "9a1afce2ab88e75c3ee49053e2a1e2221fb7ab13",
          "url": "https://github.com/maidsafe/safe_network/commit/d05e7b3a97db73cdf84f74560056abe1f087820a"
        },
        "date": 1654756732411,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10512741822,
            "range": "± 13029247803",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3913189626,
            "range": "± 998832612",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 9936883458,
            "range": "± 15534853741",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10035176815,
            "range": "± 4327502",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3160708144,
            "range": "± 726551844",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4774192673,
            "range": "± 867699703",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "8c7e42e9a91f803579426c5c5fcef14ace10fea0",
          "message": "Merge #1220\n\n1220: Chore logs ci r=joshuef a=joshuef\n\n<!--\r\nThanks for contributing to the project! We recommend you check out our \"Guide to contributing\" page if you haven't already: https://github.com/maidsafe/QA/blob/master/CONTRIBUTING.md\r\n\r\nWrite your comment below this line: -->\r\n\n\nCo-authored-by: Josh Wilson <joshuef@gmail.com>",
          "timestamp": "2022-06-09T12:01:09Z",
          "tree_id": "85d4e00844fced3f1729219f423166e116dc65cc",
          "url": "https://github.com/maidsafe/safe_network/commit/8c7e42e9a91f803579426c5c5fcef14ace10fea0"
        },
        "date": 1654781916505,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10479794704,
            "range": "± 9508515898",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3871789110,
            "range": "± 1131563189",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 9520661875,
            "range": "± 339160844",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10034979248,
            "range": "± 2216523762",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3187017857,
            "range": "± 1610643113",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4753497643,
            "range": "± 289298257",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "action@github.com",
            "name": "GitHub Action",
            "username": "actions-user"
          },
          "committer": {
            "email": "action@github.com",
            "name": "GitHub Action",
            "username": "actions-user"
          },
          "distinct": true,
          "id": "6253530bf609e214de3a04433dcc260aa71721e0",
          "message": "chore(release): sn_node-0.62.2",
          "timestamp": "2022-06-10T05:55:02Z",
          "tree_id": "5a9bbb647bd02407ee4e59ebf0abdf989ca9799e",
          "url": "https://github.com/maidsafe/safe_network/commit/6253530bf609e214de3a04433dcc260aa71721e0"
        },
        "date": 1654842564313,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 20231652440,
            "range": "± 14655170730",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3858990875,
            "range": "± 3414938380",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 10003639459,
            "range": "± 1879452794",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10034854976,
            "range": "± 6857532",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3164458851,
            "range": "± 978459638",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4771225985,
            "range": "± 1886126904",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joshuef@gmail.com",
            "name": "Josh Wilson",
            "username": "joshuef"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "fe2010fa66622cfcb52325ad6139bb1bf1783251",
          "message": "chore(node): add basic chaos to node startup\n\nrandom crashes to ensure that the node startup looping is in effect",
          "timestamp": "2022-06-10T08:56:42+02:00",
          "tree_id": "6b64bb3b66d6e3bd7195f394302fb027e0598947",
          "url": "https://github.com/maidsafe/safe_network/commit/fe2010fa66622cfcb52325ad6139bb1bf1783251"
        },
        "date": 1654846278872,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10510642671,
            "range": "± 5891273962",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3893443814,
            "range": "± 981958398",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 9872827107,
            "range": "± 13593210903",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10036579927,
            "range": "± 6221724",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 3160781763,
            "range": "± 32232380",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 4778992827,
            "range": "± 157566767",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "298662fa9d43f1f994dbdd22065b4ca67e3b7a03",
          "message": "Merge #1236\n\n1236: refactor(safeurl): add from_* functions r=bochaco a=RolandSherwin\n\nFix #1231 \n\nCo-authored-by: RolandSherwin <RolandSherwin@protonmail.com>\nCo-authored-by: bochaco <gabrielviganotti@gmail.com>",
          "timestamp": "2022-06-14T12:30:05Z",
          "tree_id": "c1986ed787811ee8fb4b953fa2173703076f3105",
          "url": "https://github.com/maidsafe/safe_network/commit/298662fa9d43f1f994dbdd22065b4ca67e3b7a03"
        },
        "date": 1655217988304,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 8072009877,
            "range": "± 10415621750",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 6422715936,
            "range": "± 8014971754",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 18551736378,
            "range": "± 12044755708",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 7586647092,
            "range": "± 2187676578",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 5359201776,
            "range": "± 133674735",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 11814575947,
            "range": "± 739716737",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "05b9b755165304c282cc415419030eee8b6a3636",
          "message": "Merge #1234\n\n1234: Enable DKG issue tracking r=Yoga07 a=joshuef\n\nThis is atop #1232 so need that in first\n\nCo-authored-by: Josh Wilson <joshuef@gmail.com>",
          "timestamp": "2022-06-15T09:43:04Z",
          "tree_id": "e431beca79da731b9a9b1e53eed378fd78cf8f64",
          "url": "https://github.com/maidsafe/safe_network/commit/05b9b755165304c282cc415419030eee8b6a3636"
        },
        "date": 1655293082843,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 29222362477,
            "range": "± 11613475216",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 6970914612,
            "range": "± 972845686",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 13925900988,
            "range": "± 313347476",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10046483683,
            "range": "± 6037088",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 6335567213,
            "range": "± 71578968",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 9332796571,
            "range": "± 1429733945",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "f9c7544f369e15fb3b6f91158ac3277656737fa4",
          "message": "Merge #1241\n\n1241: chore: upgrade blsttc to 6.0.0 r=jacderida a=jacderida\n\nThere were various other crates that had to be upgraded in this process:\r\n* secured_linked_list to v0.5.2 because it was also upgraded to reference v6.0.0 of blsttc\r\n* bls_dkg to v0.10.3 because it was also upgraded to reference v6.0.0 of blsttc\r\n* sn_consensus to v2.1.1 because it was also upgraded to reference v6.0.0 of blsttc\r\n* sn_dbc to v4.0.0 because it was also upgraded to reference v6.0.0 of blsttc\r\n\n\nCo-authored-by: Chris O'Neil <chriso83@protonmail.com>",
          "timestamp": "2022-06-15T14:31:26Z",
          "tree_id": "40c681175eabda1b649f1e22fb1300f18a0912d9",
          "url": "https://github.com/maidsafe/safe_network/commit/f9c7544f369e15fb3b6f91158ac3277656737fa4"
        },
        "date": 1655310556285,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 10786762192,
            "range": "± 10202506583",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 6791406791,
            "range": "± 71182411",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 12261230842,
            "range": "± 313595509",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10047688051,
            "range": "± 7547814",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 6223740860,
            "range": "± 60104108",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 8548453713,
            "range": "± 735927438",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joshuef@gmail.com",
            "name": "Josh Wilson",
            "username": "joshuef"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "4d1b0915ce6ab21c13d27b7be66a455e4fbd3133",
          "message": "chore: update rcgen to remove failure crate",
          "timestamp": "2022-06-17T11:24:50+02:00",
          "tree_id": "236fb5095363159dc310f8b8d19ba7309c8e5695",
          "url": "https://github.com/maidsafe/safe_network/commit/4d1b0915ce6ab21c13d27b7be66a455e4fbd3133"
        },
        "date": 1655460322437,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 19764868698,
            "range": "± 11260007911",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 6756274760,
            "range": "± 63561415",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 12412349894,
            "range": "± 1728519311",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10053092806,
            "range": "± 1264702907",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 6237747245,
            "range": "± 44482975",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 8509097606,
            "range": "± 263758084",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "1fad3d530f1e38544197672639e029a13d3e2207",
          "message": "Merge #1248\n\n1248: chore: reorder nodeacceptance cmds to inform node first of all r=joshuef a=joshuef\n\n<!--\r\nThanks for contributing to the project! We recommend you check out our \"Guide to contributing\" page if you haven't already: https://github.com/maidsafe/QA/blob/master/CONTRIBUTING.md\r\n\r\nWrite your comment below this line: -->\r\n\n\nCo-authored-by: Josh Wilson <joshuef@gmail.com>",
          "timestamp": "2022-06-17T11:40:13Z",
          "tree_id": "ea1eae7e1ba895def68d1c5a2a425681dae33b96",
          "url": "https://github.com/maidsafe/safe_network/commit/1fad3d530f1e38544197672639e029a13d3e2207"
        },
        "date": 1655473310849,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 20058391345,
            "range": "± 12713914566",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 6810429574,
            "range": "± 94502029",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 12595817853,
            "range": "± 447159693",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10048179660,
            "range": "± 1138661284",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 6250550897,
            "range": "± 52109449",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 8692578116,
            "range": "± 914074253",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "oetyng@gmail.com",
            "name": "oetyng",
            "username": "oetyng"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "d204cffdc25a08f604f3a7b97dd74c0f4181b696",
          "message": "chore: remove unused deps and enum variants\nWas made aware by a comment on the forum that there was a sled\ndep in `sn_interface`, which seemed wrong, and from there I found more.",
          "timestamp": "2022-06-20T11:31:30+02:00",
          "tree_id": "8ef3b77b59cd4af04213c6831ca9e4321959fe98",
          "url": "https://github.com/maidsafe/safe_network/commit/d204cffdc25a08f604f3a7b97dd74c0f4181b696"
        },
        "date": 1655720092022,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 20056450807,
            "range": "± 11146841575",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 6695487448,
            "range": "± 32494421",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 11207296449,
            "range": "± 13572363834",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10049252503,
            "range": "± 7580977",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 6173654853,
            "range": "± 60903630",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 8263316830,
            "range": "± 1958286212",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "cff8b337be20f3e1c0cddc5464c2eee0c8cc9e1c",
          "message": "Merge #1256\n\n1256: refactor(events): cleanup and restructure of enum r=joshuef a=oetyng\n\n - Initiates the use of the node event channel for more structured\r\nlogging.\r\nBREAKING CHANGE: events renamed and restructured\n\nCo-authored-by: oetyng <oetyng@gmail.com>\nCo-authored-by: joshuef <joshuef@gmail.com>",
          "timestamp": "2022-06-21T07:26:27Z",
          "tree_id": "a14084ec411025b82d251988bdebcc649b37703f",
          "url": "https://github.com/maidsafe/safe_network/commit/cff8b337be20f3e1c0cddc5464c2eee0c8cc9e1c"
        },
        "date": 1655802981093,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 6448838898,
            "range": "± 4152237",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 6768071655,
            "range": "± 73941548",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 12185008952,
            "range": "± 1445475729",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 6033727951,
            "range": "± 154873893",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 6227621271,
            "range": "± 40273172",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 8648234143,
            "range": "± 158106548",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "ed0b5d890e8404a59c25f8131eab5d23ce12eb7d",
          "message": "Merge #1255 #1258\n\n1255: refactor: improve efficiency of load monitoring r=joshuef a=oetyng\n\nRefactors load monitoring so that it is more efficiently used, and\r\nfor both outgoing msgs and (in coming commit) cmds.\n\n1258: ci: remove register data_storage benchmark for now as sled db keeps e… r=joshuef a=joshuef\n\n…rroring\r\n\r\n<!--\r\nThanks for contributing to the project! We recommend you check out our \"Guide to contributing\" page if you haven't already: https://github.com/maidsafe/QA/blob/master/CONTRIBUTING.md\r\n\r\nWrite your comment below this line: -->\r\n\n\nCo-authored-by: oetyng <oetyng@gmail.com>\nCo-authored-by: Josh Wilson <joshuef@gmail.com>",
          "timestamp": "2022-06-21T12:14:38Z",
          "tree_id": "90fab8af9d8e313357083fd04be88bc9238a91ab",
          "url": "https://github.com/maidsafe/safe_network/commit/ed0b5d890e8404a59c25f8131eab5d23ce12eb7d"
        },
        "date": 1655819828961,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 6447629339,
            "range": "± 159703663",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 6790362976,
            "range": "± 651310103",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 12314213220,
            "range": "± 379556423",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 6036152959,
            "range": "± 6356812",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 6215394716,
            "range": "± 29385243",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 8603512976,
            "range": "± 243124246",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joshuef@gmail.com",
            "name": "Josh Wilson",
            "username": "joshuef"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "2445103118a5b9df0056525c804b3c76313ad084",
          "message": "fix: update network_is_ready.sh for sn_client dir",
          "timestamp": "2022-06-23T08:38:34+02:00",
          "tree_id": "94f7acf5a7865db65d176578e0c2f89975f12eec",
          "url": "https://github.com/maidsafe/safe_network/commit/2445103118a5b9df0056525c804b3c76313ad084"
        },
        "date": 1655969015853,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 6452754999,
            "range": "± 235039407",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 6804647454,
            "range": "± 638008780",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 13196601194,
            "range": "± 318320465",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 6035933839,
            "range": "± 6388787",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 6284894667,
            "range": "± 53389628",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 9102600983,
            "range": "± 39023263",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "7f4f4cb0b1664c2d6f30962de25d5fdcbc5074de",
          "message": "Merge #1264\n\n1264: test: improving dysf test, reproducible issues r=Yoga07 a=joshuef\n\n<!--\r\nThanks for contributing to the project! We recommend you check out our \"Guide to contributing\" page if you haven't already: https://github.com/maidsafe/QA/blob/master/CONTRIBUTING.md\r\n\r\nWrite your comment below this line: -->\r\n\n\nCo-authored-by: Josh Wilson <joshuef@gmail.com>",
          "timestamp": "2022-06-23T08:22:56Z",
          "tree_id": "5a9f1c4803f8568162ca58aebdd2a6db5c619d83",
          "url": "https://github.com/maidsafe/safe_network/commit/7f4f4cb0b1664c2d6f30962de25d5fdcbc5074de"
        },
        "date": 1655978228277,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 6447199604,
            "range": "± 203299389",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 6764650585,
            "range": "± 85307908",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 12532109187,
            "range": "± 1452613516",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 6036782712,
            "range": "± 5083562",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 6217414112,
            "range": "± 33640502",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 8701450338,
            "range": "± 183459773",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "2415f169917f101459ec6273375dc5e2cbbd06d4",
          "message": "Merge #1261\n\n1261: feat(flow_control): organize internal work r=joshuef a=oetyng\n\n- Organizes internal work so that internal cmds (work) are now dealt\r\nwith according to priority.\r\n- Enables adaptive throughput of cmds.\r\n- Prepares for logging of cmds separately (future feat).\r\n\n\nCo-authored-by: oetyng <oetyng@gmail.com>",
          "timestamp": "2022-06-23T12:37:48Z",
          "tree_id": "bb031e1f0a321511ba34d8216a6e533a577a5d25",
          "url": "https://github.com/maidsafe/safe_network/commit/2415f169917f101459ec6273375dc5e2cbbd06d4"
        },
        "date": 1655993463865,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 6647123729,
            "range": "± 80026366",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 7121897475,
            "range": "± 1514478596",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 17892920867,
            "range": "± 799884178",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 6023935578,
            "range": "± 3901480",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 6316114008,
            "range": "± 89757513",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 12132613430,
            "range": "± 519236453",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "366be4d3ddc39f32beea0e26d0addd161acc90c2",
          "message": "Merge #1266\n\n1266: chore(misc): misc cleanup and fixes r=joshuef a=oetyng\n\n- Complete `msg_kind` => `auth_kind` renaming.\r\n- Fix broken `routing_stress` startup.\r\n- Clarify context of `HandleTimeout` and `ScheduleTimeout` by\r\ninserting `Dkg`.\r\n- Tweak `network_split` example.\r\n- Set various things, such as payload debug, under `test-utils` flag.\r\n- Fix comments/logs: the opposite group of `full` adults are\r\n`non-full`, not `empty`.\r\n\r\n<!--\r\nThanks for contributing to the project! We recommend you check out our \"Guide to contributing\" page if you haven't already: https://github.com/maidsafe/QA/blob/master/CONTRIBUTING.md\r\n\r\nWrite your comment below this line: -->\r\n\n\nCo-authored-by: oetyng <oetyng@gmail.com>",
          "timestamp": "2022-06-23T14:07:30Z",
          "tree_id": "173076c535cf2e3f90fbaf67907d4d6ebe07d4f0",
          "url": "https://github.com/maidsafe/safe_network/commit/366be4d3ddc39f32beea0e26d0addd161acc90c2"
        },
        "date": 1655998925309,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 6646502175,
            "range": "± 78963812",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 7035877544,
            "range": "± 649179448",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 15347060993,
            "range": "± 1591744905",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 6019694457,
            "range": "± 2143022",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 6239467640,
            "range": "± 72431752",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 11015379701,
            "range": "± 417709957",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "action@github.com",
            "name": "GitHub Action",
            "username": "actions-user"
          },
          "committer": {
            "email": "action@github.com",
            "name": "GitHub Action",
            "username": "actions-user"
          },
          "distinct": true,
          "id": "dc69a62eec590b2d621ab2cbc3009cb052955e66",
          "message": "chore(release): sn_interface-0.6.5/sn_dysfunction-0.5.3/sn_client-0.66.5/sn_node-0.62.8/sn_cli-0.57.6",
          "timestamp": "2022-06-24T06:03:41Z",
          "tree_id": "59814194fcb971ea65eca527ab59e2363d2b19a6",
          "url": "https://github.com/maidsafe/safe_network/commit/dc69a62eec590b2d621ab2cbc3009cb052955e66"
        },
        "date": 1656052592508,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 6707074484,
            "range": "± 81600998",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 7062364526,
            "range": "± 1069569563",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 19222359916,
            "range": "± 1403735692",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 6025132278,
            "range": "± 5649118",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 6353844710,
            "range": "± 114394231",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 12728952555,
            "range": "± 468784384",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "e9adc0d3ba2f33fe0b4590a5fe11fea56bd4bda9",
          "message": "Merge #1268\n\n1268: test: make the measurement of client bench test more accurate r=joshuef a=maqi\n\n<!--\r\nThanks for contributing to the project! We recommend you check out our \"Guide to contributing\" page if you haven't already: https://github.com/maidsafe/QA/blob/master/CONTRIBUTING.md\r\n\r\nWrite your comment below this line: -->\r\n\n\nCo-authored-by: qima <qi.ma@maidsafe.net>",
          "timestamp": "2022-06-24T08:42:22Z",
          "tree_id": "daf4365e4dbb3b3d78d88d0570c4f471d41df0f3",
          "url": "https://github.com/maidsafe/safe_network/commit/e9adc0d3ba2f33fe0b4590a5fe11fea56bd4bda9"
        },
        "date": 1656065354350,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 5527315924,
            "range": "± 93293094",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 5594343209,
            "range": "± 54441981",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 5719935745,
            "range": "± 37342900",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 5013561953,
            "range": "± 1518081",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 5021359705,
            "range": "± 1575319",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 5103673601,
            "range": "± 17072274",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joshuef@gmail.com",
            "name": "Josh Wilson",
            "username": "joshuef"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "c85dc4c7a07d2f5343490328c593cceb0f50c6aa",
          "message": "chore: more tweaks to benchmarks for clippy",
          "timestamp": "2022-06-24T16:28:55+02:00",
          "tree_id": "88bfa72a43864d3a2a25afbe57ae41c82424e71a",
          "url": "https://github.com/maidsafe/safe_network/commit/c85dc4c7a07d2f5343490328c593cceb0f50c6aa"
        },
        "date": 1656082520757,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 5543233578,
            "range": "± 133906738",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 5672067224,
            "range": "± 39342852",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 5815493225,
            "range": "± 110926844",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 5012419484,
            "range": "± 3505552",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 5028856854,
            "range": "± 2848160",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 5189193076,
            "range": "± 15611320",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "chriso83@protonmail.com",
            "name": "Chris O'Neil",
            "username": "jacderida"
          },
          "committer": {
            "email": "gabrielviganotti@gmail.com",
            "name": "bochaco",
            "username": "bochaco"
          },
          "distinct": true,
          "id": "3f3c39a14987910bb424df51f89d948333ca3e87",
          "message": "chore: changes based on review feedback\n\n* Prefer `map_err` in various places rather than a full `match`.\n* Change key serialization utility functions to static rather than instance.\n* Change `dog` command to print non-support of `SafeKey` data type rather than panic.\n* Remove unnecessary clone on `public_key_hex`.\n* Remove unnecessary match in various tests.\n* Ignore wallet CLI tests that deleted the credentials file. They are problematic when running in\n  parallel with other tests. We need better isolated testing mechanisms for these. Will address in a\n  separate PR.\n* Use different deposit names in wallet tests where multiple DBCs are deposited.",
          "timestamp": "2022-06-24T19:05:07-03:00",
          "tree_id": "d2bd5a1336d327ca2745fb67c00f710f173a62f3",
          "url": "https://github.com/maidsafe/safe_network/commit/3f3c39a14987910bb424df51f89d948333ca3e87"
        },
        "date": 1656110281306,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 5478853879,
            "range": "± 142558153",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 5658735959,
            "range": "± 66448444",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 5874866476,
            "range": "± 34979901",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 5014612104,
            "range": "± 2979550",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 5036823915,
            "range": "± 2315212",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 5235072867,
            "range": "± 20925942",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "de5f156936e4326a5494c722307608f22188455e",
          "message": "Merge #1273\n\n1273: feat: deposit owned dbcs r=jacderida a=jacderida\n\n- 554c9d688 **chore: upgrade sn_dbc to 3.3.0**\r\n\r\n  This version contains an update for converting an owned DBC to a bearer DBC.\r\n\r\n- 23802f8e3 **feat!: extend wallet_deposit for owned dbcs**\r\n\r\n  BREAKING CHANGE: the `wallet_deposit` function now provides an `Option<bls::SecretKey>` argument for\r\n  providing a secret key when depositing an owned DBC.\r\n\r\n  The owned DBC is changed to a bearer by providing the secret key, and the resulting DBC is then\r\n  stored in the wallet. This means you don't need to provide the secret key at reissue time, where\r\n  there is an issue with mapping input DBCs to secret keys.\r\n\r\n  A couple of misc changes:\r\n  * Extra test cases were added here to cover assigning the name to the deposit.\r\n  * More instances of \"Wallet\" were converted to \"wallet\", because \"wallet\" is not a proper noun.\r\n\r\n  As an unrelated change, this commit also provides more test coverage for `keys show` command. This\r\n  isn't a hugely important command, but I just wanted to ensure we don't break support for BLS keys.\r\n\r\n  These are integration tests because you really need to parse the console output, which you can't do\r\n  with unit tests.\r\n\r\n  This should probably have been its own commit, but it was accidentally added via an amend, then the\r\n  API changes were also amended on top, so it was too difficult to break out.\r\n\r\n- 67006eb2e **feat!: serialize to bls keys in util functions**\r\n\r\n  Utility functions were recently added to the API for serializing to the `Keypair` type. This was\r\n  changed to serialize directly to BLS to make it easier for the CLI to deal directly with BLS keys.\r\n  Soon we will be refactoring the `Keypair` type to have a different use case and things like\r\n  `sn_client` would be refactored to directly work with BLS keys. This is a little step in that\r\n  direction.\r\n\r\n  There was a utility function added to `sn_interface` to create a `Keypair` from a hex-based BLS key\r\n  because we still need to use the `Keypair` at this point in time.\r\n\r\n- f7940c5cd **feat!: remove use of xorurl with keys command**\r\n\r\n  The use of BLS keys means the XorUrl that was displayed to the user will no longer be usable, since\r\n  the BLS key is a 48-byte structure, and not 32 bytes like the Dalek key. We will come back and\r\n  address this issue later, possibly re-introducing the `SafeKey` type in some kind of different form.\r\n\r\n  The functionality and corresponding test cases for the `cat` and `dog` command that were working\r\n  with the `SafeKey` data type were removed.\r\n\r\n  The test cases for the `keys` commands were elaborated here to check the output of both the pretty\r\n  print and the json formats and to make sure the CLI prints out a matching keypair.\r\n\r\n- 69079d698 **feat: extend cli wallet deposit for owned dbcs**\r\n\r\n  The CLI is now extended to support the deposit of owned DBCs.\r\n\r\n  The `deposit` command will check if the supplied DBC is owned, and if it is, it will check to see if\r\n  the `--secret-key` argument is present and use that. If that argument isn't present, it will attempt\r\n  to use the secret key that's configured for use with the CLI, i.e., the `keys create --for-cli`\r\n  command.\r\n\r\n  The `reissue` command was also extended to provide an `--owned` flag, which when used, will reissue\r\n  an owned DBC using the public key configured for use with the CLI. This argument is mutually\r\n  exclusive with the `--public-key` argument, which will reissue the DBC using a specified key.\r\n\r\n  So we could offer the user a suggestion when a supplied secret key didn't match, this also involved\r\n  making a little extension to the API, to return a specific type of error. We will need to modify\r\n  `sn_dbc` to return a specific error type for this too, so we can avoid checking the string content\r\n  of the error message, but this will be covered on a separate PR.\n\nCo-authored-by: Chris O'Neil <chriso83@protonmail.com>",
          "timestamp": "2022-06-24T22:05:24Z",
          "tree_id": "d2bd5a1336d327ca2745fb67c00f710f173a62f3",
          "url": "https://github.com/maidsafe/safe_network/commit/de5f156936e4326a5494c722307608f22188455e"
        },
        "date": 1656113572791,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 5558911495,
            "range": "± 74014644",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 5606036952,
            "range": "± 52281357",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 5803860230,
            "range": "± 76310014",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 5012718913,
            "range": "± 2406725",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 5032604128,
            "range": "± 2454268",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 5205066949,
            "range": "± 19622814",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "action@github.com",
            "name": "GitHub Action",
            "username": "actions-user"
          },
          "committer": {
            "email": "action@github.com",
            "name": "GitHub Action",
            "username": "actions-user"
          },
          "distinct": true,
          "id": "243cfc48a7f4a9b60b5b7f1fdd609c02197aba5e",
          "message": "chore(release): sn_interface-0.7.0/sn_dysfunction-0.6.0/sn_client-0.67.0/sn_node-0.63.0/sn_api-0.65.0/sn_cli-0.58.0",
          "timestamp": "2022-06-26T06:07:27Z",
          "tree_id": "e29bb844422d55acc091174f35739547fe84083e",
          "url": "https://github.com/maidsafe/safe_network/commit/243cfc48a7f4a9b60b5b7f1fdd609c02197aba5e"
        },
        "date": 1656225978863,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 5596137405,
            "range": "± 168944917",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 5680352965,
            "range": "± 89409731",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 5859020280,
            "range": "± 112619678",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 5014451216,
            "range": "± 1451170",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 5038825788,
            "range": "± 4772007",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 5255386187,
            "range": "± 23906608",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joshuef@gmail.com",
            "name": "Josh Wilson",
            "username": "joshuef"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "eebbc30f5dd449b786115c37813a4554309875e0",
          "message": "test: adding new dysf test for DKG rounds",
          "timestamp": "2022-06-27T14:22:00+02:00",
          "tree_id": "f96d4ecdf629c301478176333fd177598958510f",
          "url": "https://github.com/maidsafe/safe_network/commit/eebbc30f5dd449b786115c37813a4554309875e0"
        },
        "date": 1656334689243,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 5636525846,
            "range": "± 102931494",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 5682690200,
            "range": "± 146476474",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 5821934316,
            "range": "± 114259358",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 5017254407,
            "range": "± 2763328",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 5036380556,
            "range": "± 2827820",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 5204290853,
            "range": "± 29634269",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "bzeeman@live.nl",
            "name": "Benno Zeeman",
            "username": "b-zee"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "6bfd101ed12a16f3f6a9a0b55252d45d200af7c6",
          "message": "feat(dataquery): Select which adult to query\n\nLet the client pick the adult to query, based on the XOR distance.",
          "timestamp": "2022-06-27T14:23:03+02:00",
          "tree_id": "3068fdde11295e3313773130f11da0d15e61d57c",
          "url": "https://github.com/maidsafe/safe_network/commit/6bfd101ed12a16f3f6a9a0b55252d45d200af7c6"
        },
        "date": 1656334791180,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 5777100054,
            "range": "± 80934518",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 5788794272,
            "range": "± 154304672",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 5934511394,
            "range": "± 6917541594",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 5013193772,
            "range": "± 3124429",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 5030714312,
            "range": "± 1500898",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 5200489717,
            "range": "± 23187028",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joshuef@gmail.com",
            "name": "Josh Wilson",
            "username": "joshuef"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "6cfed7b9b5c7d449424072a8acba97df3c813fc4",
          "message": "ci: ensure the split-test only starts 30 nodes",
          "timestamp": "2022-06-27T15:31:14+02:00",
          "tree_id": "1c0949cdbb2368a937c0928eca27056fda33d742",
          "url": "https://github.com/maidsafe/safe_network/commit/6cfed7b9b5c7d449424072a8acba97df3c813fc4"
        },
        "date": 1656338673797,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 5779321089,
            "range": "± 146110353",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 5782803183,
            "range": "± 159039103",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 6070400742,
            "range": "± 152734600",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 5016932457,
            "range": "± 9100722",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 5044993520,
            "range": "± 13132283",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 5468630443,
            "range": "± 165427604",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "qi.ma@maidsafe.net",
            "name": "qima",
            "username": "maqi"
          },
          "committer": {
            "email": "gabrielviganotti@gmail.com",
            "name": "bochaco",
            "username": "bochaco"
          },
          "distinct": true,
          "id": "44b93fde435214b363c009e555a2579bb3404e75",
          "message": "feat: use node's section_key and own key for register",
          "timestamp": "2022-06-27T14:34:51-03:00",
          "tree_id": "5808cc581e3d5b9678f3a98bf1c330c88c4a2ae8",
          "url": "https://github.com/maidsafe/safe_network/commit/44b93fde435214b363c009e555a2579bb3404e75"
        },
        "date": 1656352894251,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 5785815496,
            "range": "± 132578821",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 5740251042,
            "range": "± 63435321",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 5837282524,
            "range": "± 3604958479",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 5012586177,
            "range": "± 571449",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 5031565335,
            "range": "± 2079203",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 5187339608,
            "range": "± 30504022",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "action@github.com",
            "name": "GitHub Action",
            "username": "actions-user"
          },
          "committer": {
            "email": "action@github.com",
            "name": "GitHub Action",
            "username": "actions-user"
          },
          "distinct": true,
          "id": "58890e5c919ada30f27d4e80c6b5e7291b99ed5c",
          "message": "chore(release): sn_interface-0.7.1/sn_dysfunction-0.6.1/sn_client-0.67.1/sn_node-0.63.1",
          "timestamp": "2022-06-28T06:28:48Z",
          "tree_id": "85d31bd54f6999f156be6257cdf24fb6cfbdbdd2",
          "url": "https://github.com/maidsafe/safe_network/commit/58890e5c919ada30f27d4e80c6b5e7291b99ed5c"
        },
        "date": 1656399453998,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 5631464524,
            "range": "± 110830526",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 5735191213,
            "range": "± 112032031",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 5979104400,
            "range": "± 15890351247",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 5014298102,
            "range": "± 1448660",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 5033878898,
            "range": "± 3270825",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 5207542287,
            "range": "± 18518911",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "davidrusu.me@gmail.com",
            "name": "David Rusu",
            "username": "davidrusu"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "5f085f3765ab3156c74a4b7a7d7ab63a3bf6a670",
          "message": "refactor(move-comm-out-of-node): remove NodeInfo in Node struct\n\nNodeInfo store a copy of our current socket address, which is\navailable from `Comm`.\n\nThroughout our code we have to ask Comm for our current address and\nreplace the copy in NodeInfo with the address from Comm.\n\nNext changes will hopefully remove more of our reliance on Comm inside\nof Node.",
          "timestamp": "2022-06-28T09:35:23+02:00",
          "tree_id": "1b536b9b5669562217dc98d64b560eb05798846b",
          "url": "https://github.com/maidsafe/safe_network/commit/5f085f3765ab3156c74a4b7a7d7ab63a3bf6a670"
        },
        "date": 1656403401127,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 5733057304,
            "range": "± 86392378",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 5702613981,
            "range": "± 124602596",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 5893575854,
            "range": "± 9684519329",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 5014654144,
            "range": "± 3852420",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 5032037208,
            "range": "± 2042428",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 5195618218,
            "range": "± 25069464",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "bzeeman@live.nl",
            "name": "Benno Zeeman",
            "username": "b-zee"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "4e04a2b0acc79140bf1d0aefd82c0ad5b046a3cd",
          "message": "chore(misc): remove unused asyncs (clippy)\n\nUpon removing async keywords from\nsn_interface/src/network_knowledge/mod.rs a lot of removal propagated up\nand removed most of it with help of Clippy. Clippy does not yet detect\nunnecessary async in methods\n(https://github.com/rust-lang/rust-clippy/issues/9024), but will soon.\n\nWith the help of a new Clippy lint:\ncargo clippy --all-targets --all-features -- -W clippy::unused_async\nAnd automatically fixing code with:\ncargo fix --broken-code --allow-dirty --all-targets --all-features\n\nResults mostly from the single thread work of @joshuef in #1253 (and\nongoing efforts).",
          "timestamp": "2022-06-28T13:20:12+02:00",
          "tree_id": "dd119b08110646f3346bc3059af7755c776afab6",
          "url": "https://github.com/maidsafe/safe_network/commit/4e04a2b0acc79140bf1d0aefd82c0ad5b046a3cd"
        },
        "date": 1656417017446,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 5541416716,
            "range": "± 130166552",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 5815036903,
            "range": "± 135845250",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 5967578884,
            "range": "± 12149731174",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 5013313609,
            "range": "± 123644755",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 5036106405,
            "range": "± 3396666",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 5233251979,
            "range": "± 22395098",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joshuef@gmail.com",
            "name": "Josh Wilson",
            "username": "joshuef"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "3fc8df35510464e9003f6619cd4b98a929d6648a",
          "message": "chore: move Comm out of node\n\nCo-authored-by: David Rusu <david.rusu@maidsafe.net>",
          "timestamp": "2022-06-29T10:01:41+02:00",
          "tree_id": "9b96596ce869214ce2d5398d49c562ad446da004",
          "url": "https://github.com/maidsafe/safe_network/commit/3fc8df35510464e9003f6619cd4b98a929d6648a"
        },
        "date": 1656491924872,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 5635597284,
            "range": "± 126451088",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 5828704738,
            "range": "± 35938047",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 6004779661,
            "range": "± 7360273406",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 5013212434,
            "range": "± 788801",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 5033289393,
            "range": "± 2273575",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 5205697131,
            "range": "± 10449702",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "gabrielviganotti@gmail.com",
            "name": "bochaco",
            "username": "bochaco"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "5dad80d3f239f5844243fedb89f8d4baaee3b640",
          "message": "feat(dbc): have the nodes to attach valid Commitments to signed SpentProofShares\n\nBREAKING CHANGE: SpentbookCmd::Spend message now also carries the spent proofs for nodes to verify.",
          "timestamp": "2022-06-30T10:07:45+02:00",
          "tree_id": "1836ea199cdc421bbcf64fc4b35633260b2d14da",
          "url": "https://github.com/maidsafe/safe_network/commit/5dad80d3f239f5844243fedb89f8d4baaee3b640"
        },
        "date": 1656578619519,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 5239493247,
            "range": "± 6498114",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 5278490236,
            "range": "± 40629787",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 5524248028,
            "range": "± 19727983",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 5013591124,
            "range": "± 1422565",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 5033169407,
            "range": "± 4517554",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 5656577162,
            "range": "± 284626250",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "bzeeman@live.nl",
            "name": "Benno Zeeman",
            "username": "b-zee"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "2ab264744de8eeff8e26ff1423de32dadded688f",
          "message": "chore(cleanup): small cleanup tweaks to node bin",
          "timestamp": "2022-06-30T10:05:49+02:00",
          "tree_id": "a58f0ce9bf78e70a8c8e3bc9118a72bb45bad40a",
          "url": "https://github.com/maidsafe/safe_network/commit/2ab264744de8eeff8e26ff1423de32dadded688f"
        },
        "date": 1656579031444,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 5253169488,
            "range": "± 6290520",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 5294651528,
            "range": "± 55727783",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 5628842902,
            "range": "± 39366080",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 5016136987,
            "range": "± 3173382",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 5040931198,
            "range": "± 4755680",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 5289594594,
            "range": "± 16087234",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "davidrusu.me@gmail.com",
            "name": "David Rusu",
            "username": "davidrusu"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "fe75d0575b215eaa29908783e6ee9b7daa6dc455",
          "message": "fix: rename SessionStatus::Terminate to Terminating",
          "timestamp": "2022-06-30T10:05:03+02:00",
          "tree_id": "3ba314135563c7d44ab27d5f5b02b1b789523541",
          "url": "https://github.com/maidsafe/safe_network/commit/fe75d0575b215eaa29908783e6ee9b7daa6dc455"
        },
        "date": 1656579238339,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 5255661498,
            "range": "± 9593323",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 5296642012,
            "range": "± 14790503",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 5706076501,
            "range": "± 21228721163",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 5017251934,
            "range": "± 131137832",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 5045206621,
            "range": "± 6176659",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 5318820258,
            "range": "± 18575147",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "davidrusu.me@gmail.com",
            "name": "David Rusu",
            "username": "davidrusu"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "0722d8ff1f41b7611b283ae680e0993ea759058d",
          "message": "chore(clippy): replace str.push_str(format!(..)) with write!(str, ..)",
          "timestamp": "2022-07-01T07:47:07+02:00",
          "tree_id": "d99cee4e0a188b491c41c7dbf14c3711b8ae6981",
          "url": "https://github.com/maidsafe/safe_network/commit/0722d8ff1f41b7611b283ae680e0993ea759058d"
        },
        "date": 1656656748698,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 5243585927,
            "range": "± 5261007",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 5277714728,
            "range": "± 5689774",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 5013249611,
            "range": "± 1010908",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 5034032998,
            "range": "± 1234538",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 5213172117,
            "range": "± 28090470",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "davidrusu.me@gmail.com",
            "name": "David Rusu",
            "username": "davidrusu"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "9b19fd84aaf5b75676a6bc60405c88f8d18a91f8",
          "message": "chore: clippy and remove some unneccessary async",
          "timestamp": "2022-07-01T08:34:09+02:00",
          "tree_id": "a687b9658eee5ba608a59805b80f8940465ad595",
          "url": "https://github.com/maidsafe/safe_network/commit/9b19fd84aaf5b75676a6bc60405c88f8d18a91f8"
        },
        "date": 1656658817113,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 5242215284,
            "range": "± 34866410",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 5274841653,
            "range": "± 39734672",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 5492868340,
            "range": "± 17573809",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 5011549970,
            "range": "± 1554220",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 5030269040,
            "range": "± 2564183",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 5183210087,
            "range": "± 15501347",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "davidrusu.me@gmail.com",
            "name": "David Rusu",
            "username": "davidrusu"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "0722d8ff1f41b7611b283ae680e0993ea759058d",
          "message": "chore(clippy): replace str.push_str(format!(..)) with write!(str, ..)",
          "timestamp": "2022-07-01T07:47:07+02:00",
          "tree_id": "d99cee4e0a188b491c41c7dbf14c3711b8ae6981",
          "url": "https://github.com/maidsafe/safe_network/commit/0722d8ff1f41b7611b283ae680e0993ea759058d"
        },
        "date": 1656688036240,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 5246258945,
            "range": "± 35439639",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 5287373470,
            "range": "± 39242641",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 5584924898,
            "range": "± 29569087",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 5016263605,
            "range": "± 3323117",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 5038605426,
            "range": "± 5091736",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 5272021019,
            "range": "± 27813240",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "davidrusu.me@gmail.com",
            "name": "David Rusu",
            "username": "davidrusu"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "649e58a8608fb5a195160b56a29007cd3c578d57",
          "message": "fix(comm): re-enable send job retries on transient errors",
          "timestamp": "2022-07-03T10:13:30+02:00",
          "tree_id": "9f5a9f89f48eb4dd48d654dba723d474a40c2387",
          "url": "https://github.com/maidsafe/safe_network/commit/649e58a8608fb5a195160b56a29007cd3c578d57"
        },
        "date": 1656838266002,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 5240253267,
            "range": "± 4590597",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 5279942497,
            "range": "± 6014444",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 5560069026,
            "range": "± 14068900060",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 5014375072,
            "range": "± 1736415",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 5033010974,
            "range": "± 1009876",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 5220031153,
            "range": "± 16277028",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joshuef@gmail.com",
            "name": "Josh Wilson",
            "username": "joshuef"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "d4850ff81d33751ebf9e3a7c7af438f160df6e44",
          "message": "chore: clippy clea up unused",
          "timestamp": "2022-07-04T09:44:35+02:00",
          "tree_id": "0bd463b21cbf08b3197efa5f04b6706252e5cfd1",
          "url": "https://github.com/maidsafe/safe_network/commit/d4850ff81d33751ebf9e3a7c7af438f160df6e44"
        },
        "date": 1656922930161,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 5238878589,
            "range": "± 37466268",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 5269512386,
            "range": "± 4582179",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 5526212206,
            "range": "± 19422869466",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 5012848117,
            "range": "± 1051089",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 5029125899,
            "range": "± 4728606",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 5182097668,
            "range": "± 28279674",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "david.irvine@maidsafe.net",
            "name": "David Irvine",
            "username": "dirvine"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "9314a2db5dc1ae91bc4d80a65c1a8825492fc7c7",
          "message": "chore: Docs - put symbols in backticks",
          "timestamp": "2022-07-04T10:52:50+02:00",
          "tree_id": "60cc2940cfd15aa2a500533a3303d196f5d7ae95",
          "url": "https://github.com/maidsafe/safe_network/commit/9314a2db5dc1ae91bc4d80a65c1a8825492fc7c7"
        },
        "date": 1656926512587,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 5243537295,
            "range": "± 4494245",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 5282541761,
            "range": "± 8846029",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 5530708788,
            "range": "± 16110729028",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 5016103670,
            "range": "± 2209368",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 5038515554,
            "range": "± 3933020",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 5241937162,
            "range": "± 36968041",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "RolandSherwin@protonmail.com",
            "name": "RolandSherwin",
            "username": "RolandSherwin"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "da2664dd258aace2d83ea4a425cc095635e047fb",
          "message": "chore: remove node_connection_info.config from readme",
          "timestamp": "2022-07-04T13:08:55+02:00",
          "tree_id": "08261c062478cbf964b6974b5f1ea0a61b7aa812",
          "url": "https://github.com/maidsafe/safe_network/commit/da2664dd258aace2d83ea4a425cc095635e047fb"
        },
        "date": 1656934547806,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 5241280585,
            "range": "± 4460785",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 5277830352,
            "range": "± 6280847",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 5508545284,
            "range": "± 30466379",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 5016265516,
            "range": "± 274214569",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 5035296959,
            "range": "± 2253015",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 5200182106,
            "range": "± 29195909",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "action@github.com",
            "name": "GitHub Action",
            "username": "actions-user"
          },
          "committer": {
            "email": "action@github.com",
            "name": "GitHub Action",
            "username": "actions-user"
          },
          "distinct": true,
          "id": "e4e2eb56611a328806c59ed8bc80ca2567206bbb",
          "message": "chore(release): sn_interface-0.8.0/sn_dysfunction-0.7.0/sn_client-0.68.0/sn_node-0.64.0/sn_api-0.66.0/sn_cli-0.59.0",
          "timestamp": "2022-07-04T14:30:55Z",
          "tree_id": "197961c3305b22997e8df2de0cffa3eca7c1fb59",
          "url": "https://github.com/maidsafe/safe_network/commit/e4e2eb56611a328806c59ed8bc80ca2567206bbb"
        },
        "date": 1656946707662,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 5240055191,
            "range": "± 38398022",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 5274921289,
            "range": "± 8522298",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 5515843254,
            "range": "± 12780609075",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 5012969324,
            "range": "± 109804175",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 5033505866,
            "range": "± 2468627",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 5208514375,
            "range": "± 18409444",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "gabrielviganotti@gmail.com",
            "name": "bochaco",
            "username": "bochaco"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "57f635fbe80392574f7f122a9d157fbb6320c4cc",
          "message": "feat(node): generate the genesis DBC when launching first node and write it to disk",
          "timestamp": "2022-07-05T15:14:32+02:00",
          "tree_id": "fe16396b3903b7e1f2b5e449cdda04e19818cc8c",
          "url": "https://github.com/maidsafe/safe_network/commit/57f635fbe80392574f7f122a9d157fbb6320c4cc"
        },
        "date": 1657029276130,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 5241350395,
            "range": "± 7887843",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 5279501234,
            "range": "± 36199654",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 5563922505,
            "range": "± 11443241322",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 5013654275,
            "range": "± 931627",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 5038780909,
            "range": "± 1720060",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 5253734871,
            "range": "± 27286676",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "6af41dcbad76903cb5526b270100e650aa483191",
          "message": "Merge #1304\n\n1304: Less arc  r=davidrusu a=joshuef\n\n<!--\r\nThanks for contributing to the project! We recommend you check out our \"Guide to contributing\" page if you haven't already: https://github.com/maidsafe/QA/blob/master/CONTRIBUTING.md\r\n\r\nWrite your comment below this line: -->\r\n\n\nCo-authored-by: Josh Wilson <joshuef@gmail.com>\nCo-authored-by: David Rusu <davidrusu.me@gmail.com>\nCo-authored-by: qima <qi.ma@maidsafe.net>",
          "timestamp": "2022-07-05T22:40:29Z",
          "tree_id": "dd3a0b7466a98e1409151e1e6d425e6667fb3ba4",
          "url": "https://github.com/maidsafe/safe_network/commit/6af41dcbad76903cb5526b270100e650aa483191"
        },
        "date": 1657066558817,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 524050729,
            "range": "± 13648558",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 571200945,
            "range": "± 21944693",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 306075762,
            "range": "± 38137586",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 363655858,
            "range": "± 12137158",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 978640203,
            "range": "± 142946443",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joshuef@gmail.com",
            "name": "Josh Wilson",
            "username": "joshuef"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "da13669193d93b3a56fff4a956c9ac9830055a7a",
          "message": "chore: use latest sn_launch_tool release, sans StructOpt",
          "timestamp": "2022-07-06T12:24:33+02:00",
          "tree_id": "a0e5550fc2d0684295358a1711e679f6f9762e45",
          "url": "https://github.com/maidsafe/safe_network/commit/da13669193d93b3a56fff4a956c9ac9830055a7a"
        },
        "date": 1657104965403,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 523426294,
            "range": "± 11048040",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 558018512,
            "range": "± 16316498",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 1209391088,
            "range": "± 3338626114",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 306413949,
            "range": "± 6874120",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 316424528,
            "range": "± 8063163",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 647798294,
            "range": "± 263130870",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joshuef@gmail.com",
            "name": "Josh Wilson",
            "username": "joshuef"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "77cef496695e8cac9ccefccaf99cf350fb479eb9",
          "message": "chore(client): now we only contact one adult at a time increase retry count\n\nThis should get us more contact with more elders in the same amount of time as previous.\nOnly returning faster if initial adult query returns",
          "timestamp": "2022-07-06T14:08:48+02:00",
          "tree_id": "1b1afc439ceecbecf4d0b2674744b1e7970c1b24",
          "url": "https://github.com/maidsafe/safe_network/commit/77cef496695e8cac9ccefccaf99cf350fb479eb9"
        },
        "date": 1657111300108,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 510856936,
            "range": "± 10228824",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 547917263,
            "range": "± 14582699",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 1965341330,
            "range": "± 5974919400",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 295916431,
            "range": "± 3703985",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 319421154,
            "range": "± 4702423",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 590708159,
            "range": "± 5076227",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "00dc24c9263a276797b4abdff0963df5e70c4231",
          "message": "Merge #1312\n\n1312: refactor(messaging): combine handling, sending sub-modules r=joshuef a=RolandSherwin\n\nFixes #1305 \r\nExtra changes:\r\n - Make `handle_proposal` as a static method inside `Node`.\r\n- `messaging::handling::mod::handle_msg` fn moved to `messaging::mod`, rest (inside `messaging::handling::mod`) are moved to `messaging::system_msgs`\n\nCo-authored-by: RolandSherwin <RolandSherwin@protonmail.com>\nCo-authored-by: joshuef <joshuef@gmail.com>",
          "timestamp": "2022-07-06T12:11:05Z",
          "tree_id": "19682739415b5e229e71289f7fdf70e9a7448143",
          "url": "https://github.com/maidsafe/safe_network/commit/00dc24c9263a276797b4abdff0963df5e70c4231"
        },
        "date": 1657114865047,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 512028847,
            "range": "± 6975277",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 542403015,
            "range": "± 23022860",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 1235048060,
            "range": "± 6182591358",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 279894468,
            "range": "± 5791563",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 307969223,
            "range": "± 5424384",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 519965578,
            "range": "± 3993026",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "f9fa4f7857d8161e8c036cca06006bf187a6c6c3",
          "message": "Merge #1309\n\n1309: chore(rustfmt): `try!` macro is deprecated r=joshuef a=b-zee\n\nNo need for rustfmt to check/replace this, as the compiler will already\nwarn for this. Deprecated since 1.39.\n\nRemoving the option seems to trigger a couple of formatting changes that\nrustfmt did not seem to pick on before.\n\n<!--\nThanks for contributing to the project! We recommend you check out our \"Guide to contributing\" page if you haven't already: https://github.com/maidsafe/QA/blob/master/CONTRIBUTING.md\n\nWrite your comment below this line: -->\n\n\nCo-authored-by: Benno Zeeman <bzeeman@live.nl>",
          "timestamp": "2022-07-06T15:14:37Z",
          "tree_id": "0de7f524b52ffcb08a20229e6562a2b59f2b47c2",
          "url": "https://github.com/maidsafe/safe_network/commit/f9fa4f7857d8161e8c036cca06006bf187a6c6c3"
        },
        "date": 1657126086795,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 513211149,
            "range": "± 10658249",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 543562623,
            "range": "± 22655764",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 809339031,
            "range": "± 8040910858",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 281454108,
            "range": "± 2467703",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 304148491,
            "range": "± 4152965",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 516834470,
            "range": "± 6089920",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "8421959b6a80e4386c34fcd6f86a1af5044280ec",
          "message": "Merge #1308\n\n1308: cargo husky tweaks r=joshuef a=b-zee\n\n- chore(git-hook): clippy runs cargo check already\r\n- chore(git-hook): husky hook is not used\r\n\r\nThe `pre-commit` hook that was generated by Husky runs both `cargo check` and `cargo clippy`, while clippy already runs `cargo check` (kind of superset).\r\n\r\nRemove the local hook on your system, make sure husky is recompiled and generate the hook again with a test:\r\n\r\n```sh\r\nrm .git/hooks/pre-commit\r\ncargo clean --package=cargo-husky\r\ncargo test\r\n```\r\n\r\n<!--\r\nThanks for contributing to the project! We recommend you check out our \"Guide to contributing\" page if you haven't already: https://github.com/maidsafe/QA/blob/master/CONTRIBUTING.md\r\n\r\nWrite your comment below this line: -->\r\n\n\nCo-authored-by: Benno Zeeman <bzeeman@live.nl>\nCo-authored-by: joshuef <joshuef@gmail.com>",
          "timestamp": "2022-07-06T21:23:59Z",
          "tree_id": "5905ad6309d16a251f7148f46ac4029a705b6324",
          "url": "https://github.com/maidsafe/safe_network/commit/8421959b6a80e4386c34fcd6f86a1af5044280ec"
        },
        "date": 1657148118870,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 515610945,
            "range": "± 12693654",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 558610427,
            "range": "± 10453883",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 922229301,
            "range": "± 6848118090",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 301576408,
            "range": "± 4909082",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 328297520,
            "range": "± 5254754",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 607147183,
            "range": "± 10364273",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "7fe7be336799dec811c5b17e6d753ebe31e625f1",
          "message": "Merge #1313\n\n1313: feat(cmd): add parent id r=joshuef a=oetyng\n\nThis facilitates correlation in logging.\n\nCo-authored-by: oetyng <oetyng@gmail.com>\nCo-authored-by: joshuef <joshuef@gmail.com>",
          "timestamp": "2022-07-06T22:24:56Z",
          "tree_id": "7deaa7beffc843eea6531c89f8e01357428e7004",
          "url": "https://github.com/maidsafe/safe_network/commit/7fe7be336799dec811c5b17e6d753ebe31e625f1"
        },
        "date": 1657152028360,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 521307315,
            "range": "± 23895023",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 558035825,
            "range": "± 16168818",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 2008696877,
            "range": "± 4212627988",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 301263220,
            "range": "± 7598050",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 351892912,
            "range": "± 25136424",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 1039949475,
            "range": "± 80129575",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "67686f73f9e7b18bb6fbf1eadc3fd3a256285396",
          "message": "Merge #1315\n\n1315: chore(clippy): bit more low hanging clippy fruit r=davidrusu a=b-zee\n\n<!--\nThanks for contributing to the project! We recommend you check out our \"Guide to contributing\" page if you haven't already: https://github.com/maidsafe/QA/blob/master/CONTRIBUTING.md\n\nWrite your comment below this line: -->\n\n\nCo-authored-by: Benno Zeeman <bzeeman@live.nl>",
          "timestamp": "2022-07-07T00:27:00Z",
          "tree_id": "5aecbbf5ae420cce9546371e322df265da9b04a9",
          "url": "https://github.com/maidsafe/safe_network/commit/67686f73f9e7b18bb6fbf1eadc3fd3a256285396"
        },
        "date": 1657159261243,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 518459526,
            "range": "± 11204561",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 559640132,
            "range": "± 9450846",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 883870096,
            "range": "± 6225280292",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 300444462,
            "range": "± 5280609",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 326701803,
            "range": "± 2288517",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 581259497,
            "range": "± 5425973",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "action@github.com",
            "name": "GitHub Action",
            "username": "actions-user"
          },
          "committer": {
            "email": "action@github.com",
            "name": "GitHub Action",
            "username": "actions-user"
          },
          "distinct": true,
          "id": "2b00cec961561281f6b927e13e501342843f6a0f",
          "message": "chore(release): sn_interface-0.8.1/sn_dysfunction-0.7.1/sn_client-0.68.1/sn_node-0.64.1/sn_api-0.66.1/sn_cli-0.59.1",
          "timestamp": "2022-07-07T05:55:45Z",
          "tree_id": "2718a2241c320961ae4be38c296d410de7be4915",
          "url": "https://github.com/maidsafe/safe_network/commit/2b00cec961561281f6b927e13e501342843f6a0f"
        },
        "date": 1657175181794,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 512431063,
            "range": "± 12242643",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 541622963,
            "range": "± 10640779",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 1163918359,
            "range": "± 5840092016",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 278497941,
            "range": "± 5150118",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 306788731,
            "range": "± 3573847",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 542244842,
            "range": "± 3238309",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "c4d5dbf0d00c3c4d5ca4885add24627868bc825c",
          "message": "Merge #1314\n\n1314: feat(cli): display balance of the DBC that has been successfully deposited into a wallet r=bochaco a=bochaco\n\nThis also changes the `sn_api::wallet_deposit` API to return the amount deposited.\n\nCo-authored-by: bochaco <gabrielviganotti@gmail.com>",
          "timestamp": "2022-07-07T20:17:24Z",
          "tree_id": "002d9344aa85aa46cf20c8dda9945cad3188da98",
          "url": "https://github.com/maidsafe/safe_network/commit/c4d5dbf0d00c3c4d5ca4885add24627868bc825c"
        },
        "date": 1657230748738,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 515757298,
            "range": "± 16002984",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 552253317,
            "range": "± 11233584",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 1259640347,
            "range": "± 8049992774",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 292954823,
            "range": "± 6448972",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 322681083,
            "range": "± 2823864",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 555781502,
            "range": "± 14992297",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "action@github.com",
            "name": "GitHub Action",
            "username": "actions-user"
          },
          "committer": {
            "email": "action@github.com",
            "name": "GitHub Action",
            "username": "actions-user"
          },
          "distinct": true,
          "id": "b478314f331382229c9fb235dab0198f5203f509",
          "message": "chore(release): sn_api-0.66.2/sn_cli-0.59.2",
          "timestamp": "2022-07-08T05:49:05Z",
          "tree_id": "9b7ef0fc342507b696bf036fa6a4799a8d69ac2f",
          "url": "https://github.com/maidsafe/safe_network/commit/b478314f331382229c9fb235dab0198f5203f509"
        },
        "date": 1657261228125,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 521796043,
            "range": "± 17619811",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 544345331,
            "range": "± 14141900",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 1113970280,
            "range": "± 7269474644",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 280455915,
            "range": "± 5470017",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 305112144,
            "range": "± 3299476",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 548876930,
            "range": "± 2353340",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "qi.ma@maidsafe.net",
            "name": "qima",
            "username": "maqi"
          },
          "committer": {
            "email": "david.irvine@maidsafe.net",
            "name": "David Irvine",
            "username": "dirvine"
          },
          "distinct": true,
          "id": "5cff2c5325a854f04788f9111439bca75b21c60f",
          "message": "chore: ignore store_and_read_40mb as too heavy for CI",
          "timestamp": "2022-07-08T14:16:20+01:00",
          "tree_id": "eaa9b06a581871a36ca31a130abf413f750ef284",
          "url": "https://github.com/maidsafe/safe_network/commit/5cff2c5325a854f04788f9111439bca75b21c60f"
        },
        "date": 1657287860776,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 508967451,
            "range": "± 18764771",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 542483624,
            "range": "± 13652789",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 799838901,
            "range": "± 1435060556",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 284264795,
            "range": "± 52000150",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 398381795,
            "range": "± 17718616",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 1091115153,
            "range": "± 27028561",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "9c6914e2688f70a25ad5dfe74307572cb8e8fcc2",
          "message": "Merge #1301\n\n1301: feat(node): perform signature verifications on input DBC SpentProof before signing new spent proof share r=bochaco a=bochaco\n\nThis is a follow up PR (6th PR) to PR https://github.com/maidsafe/safe_network/pull/1274, PR https://github.com/maidsafe/safe_network/pull/1235, PR https://github.com/maidsafe/safe_network/pull/1097, PR https://github.com/maidsafe/safe_network/pull/1105, and PR https://github.com/maidsafe/safe_network/pull/1143.\r\n\r\n- This implements `SpentProof`s signature verification by nodes before signing for a spent proof share, as well as checking that each `SpentProof` has been signed by a known section key (by checking the sections chains).\r\n- The input key for the genesis DBC TX is changed to be the same as the genesis key and owner of the genesis DBC. This makes the genesis spent-proof TX to be signed by the genesis key, and it allows nodes to realise when the genesis DBC is the one being spent when doing the `SpentProof` public key verification described above.\r\n- Adapt client_api spentbook test to read genesis DBC from first node in testnet, by default from `~/.safe/node/local-test-network/sn-node-genesis/genesis_dbc`, unless a path is provided on `TEST_ENV_GENESIS_DBC_PATH` env var.\r\n- We temporarily allow double spents in this sn_client test. Once we have the SpentBook implementation which prevents double spents, we'll need to adapt this sn_client test to verify there is no double spent of the genesis DBC.\n\nCo-authored-by: bochaco <gabrielviganotti@gmail.com>",
          "timestamp": "2022-07-08T14:00:21Z",
          "tree_id": "30dcb65a6b5622f48ea399f4240a8d01f6c51666",
          "url": "https://github.com/maidsafe/safe_network/commit/9c6914e2688f70a25ad5dfe74307572cb8e8fcc2"
        },
        "date": 1657294111326,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 513582979,
            "range": "± 13842798",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 540335560,
            "range": "± 29011079",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 813136159,
            "range": "± 1352708466",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 279763839,
            "range": "± 41979825",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 396938676,
            "range": "± 18851096",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 1088885537,
            "range": "± 27614829",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "davidrusu.me@gmail.com",
            "name": "David Rusu",
            "username": "davidrusu"
          },
          "committer": {
            "email": "davidrusu.me@gmail.com",
            "name": "davidrusu",
            "username": "davidrusu"
          },
          "distinct": true,
          "id": "f2ab97c053f173878ae8a355454818b38e7d72a9",
          "message": "chore(inline): inline generic write_file function",
          "timestamp": "2022-07-08T12:20:14-04:00",
          "tree_id": "202a4f8e3dba666b55ceed6470d48156057cc76d",
          "url": "https://github.com/maidsafe/safe_network/commit/f2ab97c053f173878ae8a355454818b38e7d72a9"
        },
        "date": 1657298922395,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 511443765,
            "range": "± 14791513",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 531343952,
            "range": "± 16621754",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 1227001406,
            "range": "± 1360568694",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 284405874,
            "range": "± 3560333",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 359356913,
            "range": "± 44561601",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 1083624207,
            "range": "± 26079359",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "action@github.com",
            "name": "GitHub Action",
            "username": "actions-user"
          },
          "committer": {
            "email": "action@github.com",
            "name": "GitHub Action",
            "username": "actions-user"
          },
          "distinct": true,
          "id": "34bd9bd01a3f042c35e0432df2f0cfcebc32a8a8",
          "message": "chore(release): sn_interface-0.8.2/sn_client-0.68.2/sn_node-0.64.2/sn_api-0.66.3/sn_cli-0.59.3",
          "timestamp": "2022-07-10T05:52:42Z",
          "tree_id": "95a395d2b82988bc9f4564d07598eaf8aa7f0ad9",
          "url": "https://github.com/maidsafe/safe_network/commit/34bd9bd01a3f042c35e0432df2f0cfcebc32a8a8"
        },
        "date": 1657434026722,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 517049004,
            "range": "± 11184295",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 546400945,
            "range": "± 14986896",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 845215173,
            "range": "± 1350812142",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 283135342,
            "range": "± 41745227",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 396072053,
            "range": "± 18778874",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 1089435818,
            "range": "± 35573505",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "RolandSherwin@protonmail.com",
            "name": "RolandSherwin",
            "username": "RolandSherwin"
          },
          "committer": {
            "email": "david.irvine@maidsafe.net",
            "name": "David Irvine",
            "username": "dirvine"
          },
          "distinct": true,
          "id": "5523e237464a76ef682ae2dbc183692502018682",
          "message": "refactor(node): move core one level up\n\nMove `node::core` to `node`.\nRename `api` module to `node_api`\nMove `messages::mod` to `messages.rs`\nMove `create_test_max_capacity_and_root_storage` from `node::mod` to `node::cfg::mod` where it is more appropriate.",
          "timestamp": "2022-07-11T12:57:07+01:00",
          "tree_id": "5385561375cda87c5627c3f6fd877ef6dde01dbe",
          "url": "https://github.com/maidsafe/safe_network/commit/5523e237464a76ef682ae2dbc183692502018682"
        },
        "date": 1657542797627,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 513353391,
            "range": "± 10828603",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 554269490,
            "range": "± 17987911",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 832845009,
            "range": "± 792666883",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 396681505,
            "range": "± 40585581",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 398567364,
            "range": "± 7401510",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 1085640117,
            "range": "± 22988748",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "09c2fb304c3a66902d81353153d5138f3e5a0a79",
          "message": "Merge #1269\n\n1269: tests(relocate): unit tests for JoiningAsRelocated r=Yoga07 a=RolandSherwin\n\n\n\nCo-authored-by: RolandSherwin <RolandSherwin@protonmail.com>",
          "timestamp": "2022-07-11T14:08:52Z",
          "tree_id": "55894ac33c96378b0dbae093a36e360b76aeeba3",
          "url": "https://github.com/maidsafe/safe_network/commit/09c2fb304c3a66902d81353153d5138f3e5a0a79"
        },
        "date": 1657554026541,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 527560116,
            "range": "± 14141452",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 562110430,
            "range": "± 12687706",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 949584621,
            "range": "± 1258617929",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 306416753,
            "range": "± 4068392",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 332568969,
            "range": "± 3916141",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 1086263775,
            "range": "± 223830414",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "action@github.com",
            "name": "GitHub Action",
            "username": "actions-user"
          },
          "committer": {
            "email": "action@github.com",
            "name": "GitHub Action",
            "username": "actions-user"
          },
          "distinct": true,
          "id": "5068b155ce42f0902f9f3847e8069dc415910f34",
          "message": "chore(release): sn_node-0.64.3",
          "timestamp": "2022-07-12T05:50:20Z",
          "tree_id": "15a8425695ddaa4ba00327b8607995ed53d2182e",
          "url": "https://github.com/maidsafe/safe_network/commit/5068b155ce42f0902f9f3847e8069dc415910f34"
        },
        "date": 1657606795017,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 514320496,
            "range": "± 8321243",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 561073116,
            "range": "± 9317073",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 905823690,
            "range": "± 1251285032",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 303371409,
            "range": "± 3413190",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 328641099,
            "range": "± 3404018",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 1089575739,
            "range": "± 101199254",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "bzeeman@live.nl",
            "name": "Benno Zeeman",
            "username": "b-zee"
          },
          "committer": {
            "email": "davidrusu.me@gmail.com",
            "name": "davidrusu",
            "username": "davidrusu"
          },
          "distinct": true,
          "id": "4d717a21a2daf6ef0b3b5826329a8848f2fe46ee",
          "message": "style: tweak sn_client/Cargo.toml formatting TOML",
          "timestamp": "2022-07-12T09:11:27-04:00",
          "tree_id": "31c9ae2483ac4162be7b5e9cd5fb282220e48cc9",
          "url": "https://github.com/maidsafe/safe_network/commit/4d717a21a2daf6ef0b3b5826329a8848f2fe46ee"
        },
        "date": 1657633135053,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 513174101,
            "range": "± 11310200",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 538042082,
            "range": "± 14713627",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 841964315,
            "range": "± 1125374038",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 395550196,
            "range": "± 44780845",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 395749463,
            "range": "± 16131007",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 1086705710,
            "range": "± 28712416",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "ec1499d2a2ff0177b571f510c585ab71a2176cda",
          "message": "Merge #1323\n\n1323: fix(sn_client): upon receiving an AE msg update client knowledge of network sections chains r=bochaco a=bochaco\n\n\n\nCo-authored-by: bochaco <gabrielviganotti@gmail.com>",
          "timestamp": "2022-07-12T16:16:05Z",
          "tree_id": "c2fd72330433847694831b022053f3658bb960fa",
          "url": "https://github.com/maidsafe/safe_network/commit/ec1499d2a2ff0177b571f510c585ab71a2176cda"
        },
        "date": 1657647199723,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 513560017,
            "range": "± 12746698",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 541920271,
            "range": "± 23369013",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 935259211,
            "range": "± 972894715",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 282507463,
            "range": "± 37846183",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 398162964,
            "range": "± 17955712",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 1092122396,
            "range": "± 34422983",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "d67971528697627245872f167de690029735c7d7",
          "message": "Merge #1320\n\n1320: feat(cli): show the DBC owner in the wallet displayed by cat cmd r=joshuef a=bochaco\n\n- Display the owner of each DBC when `cat`-ing a wallet.\r\n- Align to the right the balance of each DBC when `cat`-ing a wallet.\r\n- Shorten the default name set to DBC when deposited in a wallet.\r\n- Make the name of the change DBC automatically deposited in the wallet unique.\r\n\r\nExample output:\r\n```\r\n$ safe cat safe://hyryynyw5jg6rw87x795yjqbaz1rc9e4w8yhqu8x45zksrb4f9h9g9ejf8yb6o \r\nSpendable balances of wallet at \"safe://hyryynyw5jg6rw87x795yjqbaz1rc9e4w8yhqu8x45zksrb4f9h9g9ejf8yb6o\":\r\n+------------------------+----------------------+-----------------+---------------------+\r\n| Spendable balance name | Balance              | Owner           | DBC Data            |\r\n|------------------------+----------------------+-----------------+---------------------|\r\n| dbc-6892553f           |          1.987000000 | 852b82...34600c | a17d3767...00000000 |\r\n|------------------------+----------------------+-----------------+---------------------|\r\n| dbc-75a447a3           |         50.123456789 | 81aa22...023193 | 40bbaf41...00000000 |\r\n|------------------------+----------------------+-----------------+---------------------|\r\n| change-dbc-dac2b72e    | 4524969502.025287444 | a63bbd...f24bd1 | 5caea6f4...00000000 |\r\n+------------------------+----------------------+-----------------+---------------------+\r\n```\n\nCo-authored-by: bochaco <gabrielviganotti@gmail.com>",
          "timestamp": "2022-07-13T07:56:29Z",
          "tree_id": "26b0eaf7b88ca8a6825caf5874649068285251e7",
          "url": "https://github.com/maidsafe/safe_network/commit/d67971528697627245872f167de690029735c7d7"
        },
        "date": 1657704840805,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 518860700,
            "range": "± 10917491",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 544632103,
            "range": "± 16973660",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 1219635992,
            "range": "± 2009057365",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 280521428,
            "range": "± 4086059",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 314062240,
            "range": "± 46767229",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 1089660528,
            "range": "± 26898436",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "RolandSherwin@protonmail.com",
            "name": "RolandSherwin",
            "username": "RolandSherwin"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "9eaf971ac4c16bf326f6443636427951f00ae2b6",
          "message": "fix(bench): enable sn_cli bench",
          "timestamp": "2022-07-13T15:17:03+02:00",
          "tree_id": "704250f235c5d0bead8b8f2612b99cd303ddd58b",
          "url": "https://github.com/maidsafe/safe_network/commit/9eaf971ac4c16bf326f6443636427951f00ae2b6"
        },
        "date": 1657720769535,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 520269746,
            "range": "± 12597292",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 557589018,
            "range": "± 14291440",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 929841945,
            "range": "± 1237149865",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 304781883,
            "range": "± 5357370",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 333128088,
            "range": "± 18307328",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 1088844722,
            "range": "± 55321028",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 5511079,
            "range": "± 263427",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "qi.ma@maidsafe.net",
            "name": "qima",
            "username": "maqi"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "a84de2a5d92ad050904f5d91e1b6712cc4fb5986",
          "message": "chore: make upload-artifact action using main",
          "timestamp": "2022-07-13T15:17:22+02:00",
          "tree_id": "cf05bf4316099fba1db9c9bb9a979c453c0bcf3e",
          "url": "https://github.com/maidsafe/safe_network/commit/a84de2a5d92ad050904f5d91e1b6712cc4fb5986"
        },
        "date": 1657720826483,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 527050010,
            "range": "± 17224837",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 556559217,
            "range": "± 10397409",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 928821047,
            "range": "± 1946813497",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 306808191,
            "range": "± 4436537",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 330527494,
            "range": "± 2536491",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 1057107214,
            "range": "± 239009514",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 5302544,
            "range": "± 127133",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "gabrielviganotti@gmail.com",
            "name": "bochaco",
            "username": "bochaco"
          },
          "committer": {
            "email": "gabrielviganotti@gmail.com",
            "name": "bochaco",
            "username": "bochaco"
          },
          "distinct": true,
          "id": "f5af444b8ac37d2debfbe5e1d4dcdc48de963694",
          "message": "refactor(api): removing hard-coded test DBC from sn_api Wallet unit tests",
          "timestamp": "2022-07-13T14:57:41-03:00",
          "tree_id": "7a51fdcf500587158b39b4fc0bad8501761cc2b2",
          "url": "https://github.com/maidsafe/safe_network/commit/f5af444b8ac37d2debfbe5e1d4dcdc48de963694"
        },
        "date": 1657737387001,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 1222650641,
            "range": "± 989806605",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 3363498858,
            "range": "± 1390672615",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 3671691140,
            "range": "± 1479226856",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 297064703,
            "range": "± 5893456",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 318438828,
            "range": "± 2985434",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 574478011,
            "range": "± 5598393",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 4705621,
            "range": "± 116428",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "bzeeman@live.nl",
            "name": "Benno Zeeman",
            "username": "b-zee"
          },
          "committer": {
            "email": "davidrusu.me@gmail.com",
            "name": "davidrusu",
            "username": "davidrusu"
          },
          "distinct": true,
          "id": "a378e7ba67ec18be708a2e1a9e08e63519da7451",
          "message": "fix(client): Remove unused Arc(RwLock) structure",
          "timestamp": "2022-07-13T16:34:54-04:00",
          "tree_id": "121ae4ae213c3f1fbd14a666ecdd8e209bcc2df9",
          "url": "https://github.com/maidsafe/safe_network/commit/a378e7ba67ec18be708a2e1a9e08e63519da7451"
        },
        "date": 1657746562003,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 514288046,
            "range": "± 17466746",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 537652673,
            "range": "± 22764863",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 1237487307,
            "range": "± 28016565",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 395720221,
            "range": "± 21478089",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 396190397,
            "range": "± 18650649",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 1093560275,
            "range": "± 21971811",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 4246323,
            "range": "± 231502",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "qi.ma@maidsafe.net",
            "name": "qima",
            "username": "maqi"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "98fd4b29738a6ed6b9439ada6151d534caa99615",
          "message": "chore: tar the log files before upload-artifact",
          "timestamp": "2022-07-14T10:40:42+02:00",
          "tree_id": "442e36a4768ab71ee515d3ce17beb552cb7d643f",
          "url": "https://github.com/maidsafe/safe_network/commit/98fd4b29738a6ed6b9439ada6151d534caa99615"
        },
        "date": 1657790138362,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 509936874,
            "range": "± 10707715",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 541860597,
            "range": "± 16916458",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 1233164512,
            "range": "± 55410704",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 396471034,
            "range": "± 22300894",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 397234855,
            "range": "± 17274688",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 1091424084,
            "range": "± 29100486",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 4287462,
            "range": "± 74950",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "davidrusu.me@gmail.com",
            "name": "David Rusu",
            "username": "davidrusu"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "4e44e373da9ee75b2563b39c26794299e607f48f",
          "message": "fix(membership): only process each membership decision once",
          "timestamp": "2022-07-14T10:39:59+02:00",
          "tree_id": "27fd74b7e129f9a8fb301722a9f0d909e8bdfaea",
          "url": "https://github.com/maidsafe/safe_network/commit/4e44e373da9ee75b2563b39c26794299e607f48f"
        },
        "date": 1657790145411,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 519290272,
            "range": "± 9916786",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 543147424,
            "range": "± 14256077",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 1231670874,
            "range": "± 45169596",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 397387186,
            "range": "± 21926255",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 398012574,
            "range": "± 15372336",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 1092477492,
            "range": "± 27005597",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 4331452,
            "range": "± 148910",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "bzeeman@live.nl",
            "name": "Benno Zeeman",
            "username": "b-zee"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "6d237e5e7d8306cb955f436910aa01ed7221cd84",
          "message": "fix(async): unused async in CLI",
          "timestamp": "2022-07-14T12:31:12+02:00",
          "tree_id": "d211bf64b6fd3a22439cc416a12a99738a78e4a1",
          "url": "https://github.com/maidsafe/safe_network/commit/6d237e5e7d8306cb955f436910aa01ed7221cd84"
        },
        "date": 1657796726271,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 526733515,
            "range": "± 12588095",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 541556799,
            "range": "± 13523682",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 1229319795,
            "range": "± 105134498",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 397094237,
            "range": "± 22812252",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 396721143,
            "range": "± 16400906",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 1088469317,
            "range": "± 29227988",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 4080436,
            "range": "± 123591",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "bzeeman@live.nl",
            "name": "Benno Zeeman",
            "username": "b-zee"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "db525193bed7662c5184810f18587abb0d22b26b",
          "message": "fix(query-adult): use Eyre instead of boxed error",
          "timestamp": "2022-07-14T14:56:56+02:00",
          "tree_id": "83ead56f1a52862a0f6bbe24148527e4b6084e09",
          "url": "https://github.com/maidsafe/safe_network/commit/db525193bed7662c5184810f18587abb0d22b26b"
        },
        "date": 1657805532492,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 510715556,
            "range": "± 10414959",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 545312593,
            "range": "± 13839568",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 1241303544,
            "range": "± 64072878",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 398062303,
            "range": "± 22679317",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 399112546,
            "range": "± 15723989",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 1091419476,
            "range": "± 40512106",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 4518485,
            "range": "± 109886",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joshuef@gmail.com",
            "name": "Josh Wilson",
            "username": "joshuef"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "a856d788131ef85414ee1f42a868abcbbfc0d2b6",
          "message": "chore: use timestamp for log files\n\nThis means that once written a log file's name will not change.\nThis hould make debugging a live network easier (your log file is less likely to change while viewing), and also pulling logs from a network via rsync fafster (as the namess/content aren't changing all the time).",
          "timestamp": "2022-07-14T16:27:39+02:00",
          "tree_id": "b3663253480ddd079ce539cca7d6813f2622df0e",
          "url": "https://github.com/maidsafe/safe_network/commit/a856d788131ef85414ee1f42a868abcbbfc0d2b6"
        },
        "date": 1657810968746,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 511098331,
            "range": "± 10133451",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 540884664,
            "range": "± 20268035",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 1229029722,
            "range": "± 38391980",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 395797007,
            "range": "± 26803768",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 395836868,
            "range": "± 16217588",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 1088233218,
            "range": "± 28232198",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 4013519,
            "range": "± 109719",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "qi.ma@maidsafe.net",
            "name": "qima",
            "username": "maqi"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "7e2c4514fa2d923e98c640a012da7e4851bc196a",
          "message": "chore: log dysfunction during CI split test",
          "timestamp": "2022-07-14T16:23:50+02:00",
          "tree_id": "32dc05f6887d6d4dde68dd87dc564120d1cf0bf5",
          "url": "https://github.com/maidsafe/safe_network/commit/7e2c4514fa2d923e98c640a012da7e4851bc196a"
        },
        "date": 1657810982737,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 527360340,
            "range": "± 18928399",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 552946650,
            "range": "± 12029763",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 1220162941,
            "range": "± 148650318",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 396036623,
            "range": "± 30044036",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 394991545,
            "range": "± 24762509",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 1091489219,
            "range": "± 55035181",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 5333224,
            "range": "± 159511",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "qi.ma@maidsafe.net",
            "name": "qima",
            "username": "maqi"
          },
          "committer": {
            "email": "davidrusu.me@gmail.com",
            "name": "davidrusu",
            "username": "davidrusu"
          },
          "distinct": true,
          "id": "1a15d83e29648034be54fdf32663e36ce86c6de3",
          "message": "fix: resolve script error for tar",
          "timestamp": "2022-07-14T13:03:43-04:00",
          "tree_id": "e26bdd8f67cdc83a7968b68af1c5a6605f4f61c9",
          "url": "https://github.com/maidsafe/safe_network/commit/1a15d83e29648034be54fdf32663e36ce86c6de3"
        },
        "date": 1657820697609,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 3230699224,
            "range": "± 2713035940",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 8899595499,
            "range": "± 3379842188",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 3516331079,
            "range": "± 2894769413",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 239482642,
            "range": "± 121872875",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 395770184,
            "range": "± 28257883",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 1085300139,
            "range": "± 36141192",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 5400111,
            "range": "± 262697",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "chriso83@protonmail.com",
            "name": "Chris O'Neil",
            "username": "jacderida"
          },
          "committer": {
            "email": "chris.oneil@gmail.com",
            "name": "Chris O'Neil",
            "username": "jacderida"
          },
          "distinct": true,
          "id": "4521dafc707448b8887194bde4a923a1c23f790e",
          "message": "fix: copy genesis dbc to correct location",
          "timestamp": "2022-07-15T18:46:47+01:00",
          "tree_id": "f0a00fd1589f6d9551218b9d9a3dd30660913b69",
          "url": "https://github.com/maidsafe/safe_network/commit/4521dafc707448b8887194bde4a923a1c23f790e"
        },
        "date": 1657909486996,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 529459869,
            "range": "± 7508491",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 556806848,
            "range": "± 11999551",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 1208338817,
            "range": "± 127107713",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 396169804,
            "range": "± 31007061",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 394903126,
            "range": "± 22162977",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 1090732460,
            "range": "± 30462785",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 5287922,
            "range": "± 90093",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "qi.ma@maidsafe.net",
            "name": "qima",
            "username": "maqi"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "753cf5910032d6a1e78461ab383e6657f5fc33c1",
          "message": "fix: resolve script error for tar",
          "timestamp": "2022-07-15T21:03:39+02:00",
          "tree_id": "c62d316beee2d6ca3c402784389ee59be0beed4c",
          "url": "https://github.com/maidsafe/safe_network/commit/753cf5910032d6a1e78461ab383e6657f5fc33c1"
        },
        "date": 1657913891397,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 517746132,
            "range": "± 10567346",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 539282561,
            "range": "± 21430601",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 1235350593,
            "range": "± 48753028",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 396593403,
            "range": "± 20209935",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 394624647,
            "range": "± 17856733",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 1088600916,
            "range": "± 29524777",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 4146347,
            "range": "± 73557",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joshuef@gmail.com",
            "name": "Josh Wilson",
            "username": "joshuef"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "2f38be726cf493c89d452b6faa50ab8284048798",
          "message": "chore(dsyfunction): relax knowledge penalty.\n\nWe've seen some CI nodes being booted due to knowledge issues, so relaxing\nthis should help there'",
          "timestamp": "2022-07-18T10:15:31+02:00",
          "tree_id": "20fdd682653685c3b49d64ab128975d7979d40e1",
          "url": "https://github.com/maidsafe/safe_network/commit/2f38be726cf493c89d452b6faa50ab8284048798"
        },
        "date": 1658134765506,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 656821366,
            "range": "± 11693181",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 688157870,
            "range": "± 19156207",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 928044411,
            "range": "± 115701739",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 373870326,
            "range": "± 44038829",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 397204666,
            "range": "± 11598021",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 1090242615,
            "range": "± 20872002",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 4092580,
            "range": "± 72631",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "oetyng@gmail.com",
            "name": "oetyng",
            "username": "oetyng"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "38d25d6df71e3bb71e8efda50a4bf64345f69f81",
          "message": "fix(tests): update for the split of HandleCmd\nAlso removes order requirement of resulting cmds in one test,\nas the order is not a system requirement.",
          "timestamp": "2022-07-18T10:15:17+02:00",
          "tree_id": "594098bc2bcf62cf18c0510eaed1b0af1f1df2c9",
          "url": "https://github.com/maidsafe/safe_network/commit/38d25d6df71e3bb71e8efda50a4bf64345f69f81"
        },
        "date": 1658134821253,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 660048398,
            "range": "± 13226729",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 670291890,
            "range": "± 20929031",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 942837913,
            "range": "± 9673798",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 391812507,
            "range": "± 40106940",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 398077489,
            "range": "± 10133315",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 1089533782,
            "range": "± 28098858",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 4152377,
            "range": "± 168383",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "f664d797b56e7cbf03893c98d6c27d9c6d882be4",
          "message": "Merge #1324 #1345 #1346\n\n1324: feat(msgs): validate w low prio before handling r=joshuef a=oetyng\n\nDeserializes and validates at lower prio, to cut off an overload attack angle.\r\nAfter validity of msg has been established, the msg is dealt with according to its type prio.\n\n1345: Chore refactor send msg cmd r=joshuef a=joshuef\n\ntweaks to reduce SendMsg cmd load.\r\n\r\nas https://github.com/maidsafe/safe_network/pull/1342 but without the change to the Sendmsg priority\n\n1346: chore(dsyfunction): relax knowledge penalty. r=Yoga07 a=joshuef\n\nWe've seen some CI nodes being booted due to knowledge issues, so relaxing\r\nthis should help there'\r\n\r\n<!--\r\nThanks for contributing to the project! We recommend you check out our \"Guide to contributing\" page if you haven't already: https://github.com/maidsafe/QA/blob/master/CONTRIBUTING.md\r\n\r\nWrite your comment below this line: -->\r\n\n\nCo-authored-by: oetyng <oetyng@gmail.com>\nCo-authored-by: Josh Wilson <joshuef@gmail.com>",
          "timestamp": "2022-07-18T08:32:13Z",
          "tree_id": "2a828f22d662daa3d405e4efed03b496797069b7",
          "url": "https://github.com/maidsafe/safe_network/commit/f664d797b56e7cbf03893c98d6c27d9c6d882be4"
        },
        "date": 1658138092249,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 630070197,
            "range": "± 44154038",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 685094344,
            "range": "± 22088645",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 941480883,
            "range": "± 64570005",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 395276651,
            "range": "± 38705383",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 394877977,
            "range": "± 20567884",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 1088208683,
            "range": "± 34767689",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 4089119,
            "range": "± 50040",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "yogeshwar_1997@hotmail.com",
            "name": "Yoga07",
            "username": "Yoga07"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "5a121c19d395130e40df0134be36e4264b60972a",
          "message": "chore(benches): fix benchamrking groups in data_storage bench",
          "timestamp": "2022-07-18T12:48:00+02:00",
          "tree_id": "d9631d94f1f9552896ce93def801223108527af2",
          "url": "https://github.com/maidsafe/safe_network/commit/5a121c19d395130e40df0134be36e4264b60972a"
        },
        "date": 1658145195122,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 9460799789,
            "range": "± 3834769287",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 13329448772,
            "range": "± 3509884070",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 16861806389,
            "range": "± 2444320276",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10309967810,
            "range": "± 25370079",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 10355139893,
            "range": "± 43676434",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 10875297258,
            "range": "± 284578118",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 263520594,
            "range": "± 48284155",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 2241491141,
            "range": "± 392248549",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 9383613458,
            "range": "± 380143226",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 174736207,
            "range": "± 66112535",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 1750750207,
            "range": "± 419980028",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 5295285719,
            "range": "± 501845487",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 11702888,
            "range": "± 340995",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 107119629,
            "range": "± 22281801",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 390209836,
            "range": "± 27435781",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 12613364,
            "range": "± 372730",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 107349439,
            "range": "± 4609624",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 373517069,
            "range": "± 17069791",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 4861985,
            "range": "± 114875",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "bzeeman@live.nl",
            "name": "Benno Zeeman",
            "username": "b-zee"
          },
          "committer": {
            "email": "davidrusu.me@gmail.com",
            "name": "davidrusu",
            "username": "davidrusu"
          },
          "distinct": true,
          "id": "39c3fdf4128462e5f7c5fec3c628d394f505e2f2",
          "message": "chore(dependencies): remove unused console-subscriber",
          "timestamp": "2022-07-18T17:30:47-04:00",
          "tree_id": "d6b7323f17d6e8e2e4e87289bb0929ed3e6d6fee",
          "url": "https://github.com/maidsafe/safe_network/commit/39c3fdf4128462e5f7c5fec3c628d394f505e2f2"
        },
        "date": 1658182195632,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 658322616,
            "range": "± 20715294",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 688477186,
            "range": "± 27018325",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 944795649,
            "range": "± 14285547",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 380779904,
            "range": "± 44491764",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 397682886,
            "range": "± 18766234",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 1092469331,
            "range": "± 25953018",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 164089423,
            "range": "± 51083335",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 2001713918,
            "range": "± 342186876",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 7748949958,
            "range": "± 553396159",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 108669997,
            "range": "± 93863651",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 1345396989,
            "range": "± 407191655",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 2961533040,
            "range": "± 1053160214",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 10659046,
            "range": "± 206244",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 93337977,
            "range": "± 4671787",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 323402461,
            "range": "± 26644889",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 10721000,
            "range": "± 180864",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 94952070,
            "range": "± 6347773",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 376507654,
            "range": "± 29301492",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 4268329,
            "range": "± 61587",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "d9b46080ac849cac259983dc80b4b879e58c13ba",
          "message": "Merge #1356\n\n1356: fix(node): drop RwLock guards after job is done r=davidrusu a=b-zee\n\n<!--\nThanks for contributing to the project! We recommend you check out our \"Guide to contributing\" page if you haven't already: https://github.com/maidsafe/QA/blob/master/CONTRIBUTING.md\n\nWrite your comment below this line: -->\n\n\nCo-authored-by: Benno Zeeman <bzeeman@live.nl>",
          "timestamp": "2022-07-18T23:41:10Z",
          "tree_id": "0e81812e0a911b93fabb212e859d780cd1b87c73",
          "url": "https://github.com/maidsafe/safe_network/commit/d9b46080ac849cac259983dc80b4b879e58c13ba"
        },
        "date": 1658194070885,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 11910047058,
            "range": "± 4071881540",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 12026873325,
            "range": "± 4706944182",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 1014063287,
            "range": "± 167655483",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 313437137,
            "range": "± 1174921",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 333061587,
            "range": "± 2082045",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 586427458,
            "range": "± 7476555",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 262119091,
            "range": "± 62689376",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 2831313536,
            "range": "± 547743758",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 11165165362,
            "range": "± 793961040",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 170537683,
            "range": "± 41811634",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 1340913551,
            "range": "± 402832421",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 3935573248,
            "range": "± 152962026",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 17348662,
            "range": "± 454990",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 146647344,
            "range": "± 5612625",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 589650144,
            "range": "± 62385025",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 18434351,
            "range": "± 556802",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 148553503,
            "range": "± 5543916",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 641088119,
            "range": "± 60280285",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 6066426,
            "range": "± 167582",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "oetyng@gmail.com",
            "name": "oetyng",
            "username": "oetyng"
          },
          "committer": {
            "email": "davidrusu.me@gmail.com",
            "name": "davidrusu",
            "username": "davidrusu"
          },
          "distinct": true,
          "id": "f0d1abf6dd8731310b7749cd6cc7077886215997",
          "message": "fix(proposal): remove redundant generation field",
          "timestamp": "2022-07-18T22:28:20-04:00",
          "tree_id": "f31aa5e9f0e67a7edddd8f9d5ba9b7f36c1a7478",
          "url": "https://github.com/maidsafe/safe_network/commit/f0d1abf6dd8731310b7749cd6cc7077886215997"
        },
        "date": 1658200183926,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 652313243,
            "range": "± 22436114",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 698284792,
            "range": "± 19356147",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 988348462,
            "range": "± 86781794",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 312503649,
            "range": "± 8199939",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 396109861,
            "range": "± 18750306",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 1088889945,
            "range": "± 28241292",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 204334435,
            "range": "± 50101060",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 2109531067,
            "range": "± 386327156",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 8977310009,
            "range": "± 1052984162",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 219873184,
            "range": "± 62050882",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 815334533,
            "range": "± 101429195",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 3100272253,
            "range": "± 182436055",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 16338282,
            "range": "± 245783",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 129815064,
            "range": "± 14018501",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 468917468,
            "range": "± 37036796",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 16493725,
            "range": "± 225618",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 131679166,
            "range": "± 6980616",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 450425131,
            "range": "± 43367043",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 5835192,
            "range": "± 174218",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joshuef@gmail.com",
            "name": "Josh Wilson",
            "username": "joshuef"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "feaca15b7c44297c16a4665ceec738226bb860ba",
          "message": "chore: increase Send priority, and tweak depending on DstLocation",
          "timestamp": "2022-07-19T19:49:50+02:00",
          "tree_id": "78d80787643492dc44ea1519465526c3a1f84486",
          "url": "https://github.com/maidsafe/safe_network/commit/feaca15b7c44297c16a4665ceec738226bb860ba"
        },
        "date": 1658255394458,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 632523487,
            "range": "± 15449306",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 667595458,
            "range": "± 12066387",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 901229876,
            "range": "± 9409756",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 395320599,
            "range": "± 37331124",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 396331623,
            "range": "± 19508429",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 1089697059,
            "range": "± 21717972",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 235172695,
            "range": "± 37948464",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 2247562733,
            "range": "± 585488404",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 9808894891,
            "range": "± 849861082",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 138025614,
            "range": "± 43341898",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 859956706,
            "range": "± 451583516",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 2684449977,
            "range": "± 200746681",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 15335143,
            "range": "± 237612",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 119558569,
            "range": "± 12027909",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 420525895,
            "range": "± 25541287",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 14800261,
            "range": "± 275089",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 122800046,
            "range": "± 2105256",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 463519698,
            "range": "± 25409250",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 4790895,
            "range": "± 233316",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "RolandSherwin@protonmail.com",
            "name": "RolandSherwin",
            "username": "RolandSherwin"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "b38b7298aa1061e6f5f4df3c5b0ea3d7586d74b6",
          "message": "chore(cli bench): split PUT,CAT benches",
          "timestamp": "2022-07-20T13:35:29+02:00",
          "tree_id": "804eb1b3169e38d7d864e66e834ce379f7c762b2",
          "url": "https://github.com/maidsafe/safe_network/commit/b38b7298aa1061e6f5f4df3c5b0ea3d7586d74b6"
        },
        "date": 1658319927889,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 633545884,
            "range": "± 26568814",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 680148718,
            "range": "± 14940657",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 926177753,
            "range": "± 80069335",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 396311700,
            "range": "± 41736030",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 398215783,
            "range": "± 10105953",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 1090320078,
            "range": "± 21008911",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 231607262,
            "range": "± 43849347",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 2388848453,
            "range": "± 628380050",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 8543556338,
            "range": "± 1543629375",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 103860624,
            "range": "± 90572536",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 948077367,
            "range": "± 329136209",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 2858081667,
            "range": "± 102008133",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 13101119,
            "range": "± 1059925",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 113543808,
            "range": "± 8501081",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 441017471,
            "range": "± 26813983",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 14305687,
            "range": "± 902143",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 127393597,
            "range": "± 66115193",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 463417345,
            "range": "± 27796464",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 4765909,
            "range": "± 102007",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "39e43415bd4da8298919640e4102dc41be3f8bed",
          "message": "Merge #1364\n\n1364: refactor node bin logging r=joshuef a=b-zee\n\nSame as #1352 but without the OTLP feature...\r\n\r\n- refactor: move sn_node binary into own dir\r\n- refactor(node): move log rotater into own module\r\n- refactor(node): further modularize logging\r\n- refactor(node): split logging init into functions\r\n- refactor(node): remove cfg() directives\r\n- refactor(node): allow more log layers in future\r\n\r\n<!--\r\nThanks for contributing to the project! We recommend you check out our \"Guide to contributing\" page if you haven't already: https://github.com/maidsafe/QA/blob/master/CONTRIBUTING.md\r\n\r\nWrite your comment below this line: -->\r\n\n\nCo-authored-by: Benno Zeeman <bzeeman@live.nl>",
          "timestamp": "2022-07-20T12:38:50Z",
          "tree_id": "13d2210c281ba6bcee1987a13378cf1c2b2fe825",
          "url": "https://github.com/maidsafe/safe_network/commit/39e43415bd4da8298919640e4102dc41be3f8bed"
        },
        "date": 1658326583446,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 633048312,
            "range": "± 18532109",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 695798696,
            "range": "± 17944147",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 959416966,
            "range": "± 60437171",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 312097831,
            "range": "± 44160983",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 396765743,
            "range": "± 23177343",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 1091643358,
            "range": "± 30982273",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 228077515,
            "range": "± 55014769",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 2403028948,
            "range": "± 486510879",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 9399943731,
            "range": "± 867269826",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 112344072,
            "range": "± 29162712",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 1242926450,
            "range": "± 218098273",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 3485055153,
            "range": "± 125948462",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 17070000,
            "range": "± 736177",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 137816225,
            "range": "± 8088244",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 488777447,
            "range": "± 42123934",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 17098153,
            "range": "± 653223",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 143018819,
            "range": "± 13787236",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 513473727,
            "range": "± 24417171",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 5987285,
            "range": "± 171208",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "gabrielviganotti@gmail.com",
            "name": "bochaco",
            "username": "bochaco"
          },
          "committer": {
            "email": "gabrielviganotti@gmail.com",
            "name": "bochaco",
            "username": "bochaco"
          },
          "distinct": true,
          "id": "d4be0cc431947b035046cc4d56642a81c0880924",
          "message": "test(wallet): additional tests in sn-api for DBC verification failures",
          "timestamp": "2022-07-20T11:35:01-03:00",
          "tree_id": "4a7799d535a70f56e431e1a590f932e651de2fe5",
          "url": "https://github.com/maidsafe/safe_network/commit/d4be0cc431947b035046cc4d56642a81c0880924"
        },
        "date": 1658330805047,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 658713971,
            "range": "± 82657230",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 701537330,
            "range": "± 14825214",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 991533945,
            "range": "± 93564533",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 311490031,
            "range": "± 6406761",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 396075406,
            "range": "± 24351230",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 1087264378,
            "range": "± 32604789",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 175299560,
            "range": "± 25780352",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 2391997357,
            "range": "± 481227186",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 10003062034,
            "range": "± 1054174023",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 82641242,
            "range": "± 42080454",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 874040772,
            "range": "± 250485423",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 3328881585,
            "range": "± 188880908",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 16514525,
            "range": "± 640196",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 135793222,
            "range": "± 13069301",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 479415760,
            "range": "± 46586983",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 16062296,
            "range": "± 782135",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 140152462,
            "range": "± 11626508",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 577888301,
            "range": "± 53536123",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 5876859,
            "range": "± 599786",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "bzeeman@live.nl",
            "name": "Benno Zeeman",
            "username": "b-zee"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "01ea9c0bdb88da4f181d8f1638f2f2ad692d0ca3",
          "message": "feat(node): add OTLP support to sn_node bin",
          "timestamp": "2022-07-21T10:40:40+02:00",
          "tree_id": "92b12c42aff8a102af8608c7f0cc145a5538e26c",
          "url": "https://github.com/maidsafe/safe_network/commit/01ea9c0bdb88da4f181d8f1638f2f2ad692d0ca3"
        },
        "date": 1658395943781,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 646900357,
            "range": "± 27554901",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 669624469,
            "range": "± 15122299",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 889692963,
            "range": "± 34006959",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 396822827,
            "range": "± 16735630",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 396695572,
            "range": "± 6882844",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 1090526104,
            "range": "± 23463321",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 243613399,
            "range": "± 54152283",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 2293721135,
            "range": "± 438580397",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 9380554046,
            "range": "± 1489919730",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 82330261,
            "range": "± 29846783",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 1248571615,
            "range": "± 316988635",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 2776776785,
            "range": "± 188417807",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 14403260,
            "range": "± 303283",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 120978151,
            "range": "± 7312478",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 417571187,
            "range": "± 27433815",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 14642537,
            "range": "± 254390",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 119873321,
            "range": "± 5139254",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 466158110,
            "range": "± 34968629",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 4888579,
            "range": "± 158804",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "yogeshwar_1997@hotmail.com",
            "name": "Yoga07",
            "username": "Yoga07"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "8b3c4eb06fa988dc97b0cb75ed615ec69af29a48",
          "message": "chore: add traceroute feature to testnet bin",
          "timestamp": "2022-07-21T15:56:35+02:00",
          "tree_id": "e83284a7220949f43f24a2a862254f29fd30fe6d",
          "url": "https://github.com/maidsafe/safe_network/commit/8b3c4eb06fa988dc97b0cb75ed615ec69af29a48"
        },
        "date": 1658426328976,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 664667258,
            "range": "± 29227476",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 706955584,
            "range": "± 16503491",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 1028336484,
            "range": "± 74574741",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 314438216,
            "range": "± 1652991",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 340985550,
            "range": "± 21832904",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 1086340403,
            "range": "± 40968047",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 271904234,
            "range": "± 34315912",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 2774722299,
            "range": "± 606486708",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 10814505241,
            "range": "± 712980918",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 188913658,
            "range": "± 88471311",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 1623019270,
            "range": "± 312503868",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 3986027824,
            "range": "± 175629567",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 19693565,
            "range": "± 1397120",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 169144132,
            "range": "± 26345959",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 588575830,
            "range": "± 67279960",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 18927461,
            "range": "± 731388",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 159908678,
            "range": "± 8749449",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 671030079,
            "range": "± 57314088",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 6938091,
            "range": "± 275924",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joshuef@gmail.com",
            "name": "Josh Wilson",
            "username": "joshuef"
          },
          "committer": {
            "email": "davidrusu.me@gmail.com",
            "name": "davidrusu",
            "username": "davidrusu"
          },
          "distinct": true,
          "id": "7d12399edec6c1191c521528c1d569afc96bca99",
          "message": "Revert \"chore: Cleanup non-joined member sessions, regardless of connected state.\"\n\nThis reverts commit 934bf6cbc86e252eb3859c757a0b66c02f7826d9.",
          "timestamp": "2022-07-23T15:34:25-04:00",
          "tree_id": "95f5f3f81750c6616220ff1f27263600b97fb139",
          "url": "https://github.com/maidsafe/safe_network/commit/7d12399edec6c1191c521528c1d569afc96bca99"
        },
        "date": 1658607582983,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 660135362,
            "range": "± 48825095",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 698838954,
            "range": "± 22565910",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 963290486,
            "range": "± 16825776",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 311124542,
            "range": "± 42641046",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 396324551,
            "range": "± 14079012",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 1092420155,
            "range": "± 16277745",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 245642864,
            "range": "± 37293368",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 1789435541,
            "range": "± 471734430",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 8649530761,
            "range": "± 1715574253",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 89547528,
            "range": "± 33243820",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 946007312,
            "range": "± 211279109",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 3004173154,
            "range": "± 134433789",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 14887596,
            "range": "± 453980",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 125919011,
            "range": "± 4325325",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 460461885,
            "range": "± 28033759",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 14747078,
            "range": "± 440923",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 127706390,
            "range": "± 9477653",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 492588525,
            "range": "± 32278333",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 5493418,
            "range": "± 244641",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "davidrusu.me@gmail.com",
            "name": "David Rusu",
            "username": "davidrusu"
          },
          "committer": {
            "email": "davidrusu.me@gmail.com",
            "name": "davidrusu",
            "username": "davidrusu"
          },
          "distinct": true,
          "id": "bf2902c18b900b8b4a8abae5f966d1e08d547910",
          "message": "chore: whitespace + typo fix",
          "timestamp": "2022-07-23T17:55:18-04:00",
          "tree_id": "2da56f467908bd1889d00588d887a850595d6e21",
          "url": "https://github.com/maidsafe/safe_network/commit/bf2902c18b900b8b4a8abae5f966d1e08d547910"
        },
        "date": 1658616328157,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 665722861,
            "range": "± 26589582",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 704244495,
            "range": "± 20113503",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 992190797,
            "range": "± 25326501",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 312737545,
            "range": "± 1726169",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 394674092,
            "range": "± 27498685",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 1088311852,
            "range": "± 27764423",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 233615812,
            "range": "± 33524902",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 2668646646,
            "range": "± 199999641",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 11341984865,
            "range": "± 1462226390",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 136738975,
            "range": "± 89411867",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 1179235197,
            "range": "± 233791296",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 3528254334,
            "range": "± 234680640",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 15917959,
            "range": "± 1010862",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 144529490,
            "range": "± 27861745",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 507325445,
            "range": "± 59940062",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 16168552,
            "range": "± 1143709",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 146067276,
            "range": "± 16710495",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 601718173,
            "range": "± 54168969",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 6340085,
            "range": "± 336002",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "gabrielviganotti@gmail.com",
            "name": "bochaco",
            "username": "bochaco"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "042503a7d44f94ed3f0ce482984744e175d7752b",
          "message": "refactor(node): check for dst location to send to clients instead of msg authority type",
          "timestamp": "2022-07-25T09:04:47+02:00",
          "tree_id": "9e9282485ec04fc3f592c36ee297aaa4f7b6df9c",
          "url": "https://github.com/maidsafe/safe_network/commit/042503a7d44f94ed3f0ce482984744e175d7752b"
        },
        "date": 1658736011226,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 642002152,
            "range": "± 115319519",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 697715903,
            "range": "± 20096643",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 965089828,
            "range": "± 87418064",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 306995967,
            "range": "± 44800014",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 397377925,
            "range": "± 9667595",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 1093491967,
            "range": "± 27538907",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 217709974,
            "range": "± 38467986",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 2205015599,
            "range": "± 408664515",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 9471240430,
            "range": "± 1533783348",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 363284756,
            "range": "± 128219973",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 2357857797,
            "range": "± 420514074",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 10525411539,
            "range": "± 1201912933",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 14990703,
            "range": "± 204272",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 126186180,
            "range": "± 36080259",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 458469273,
            "range": "± 61083205",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 14620163,
            "range": "± 550675",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 124097180,
            "range": "± 4500563",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 464566504,
            "range": "± 29406565",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 5211718,
            "range": "± 119920",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "oetyng@gmail.com",
            "name": "oetyng",
            "username": "oetyng"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "6ca512e7adc6681bc85122847fa8ac08550cfb09",
          "message": "fix(datastorage): remove unused get mut access\nGets/Reads should not mutate, and nothing here did require mut access,\nso it could be removed.",
          "timestamp": "2022-07-25T09:04:30+02:00",
          "tree_id": "ef0b978850cc96c3cc617505ea2fba62480eda06",
          "url": "https://github.com/maidsafe/safe_network/commit/6ca512e7adc6681bc85122847fa8ac08550cfb09"
        },
        "date": 1658736561965,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 657438641,
            "range": "± 15340175",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 708745519,
            "range": "± 16469940",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 1035329156,
            "range": "± 14289065",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 314292265,
            "range": "± 1621682",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 356430584,
            "range": "± 26913323",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 1097039291,
            "range": "± 36342676",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 319977102,
            "range": "± 30941765",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 2678060663,
            "range": "± 583391599",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 11889249965,
            "range": "± 1608623931",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 233629123,
            "range": "± 129924463",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 2706931099,
            "range": "± 679530832",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 10139933675,
            "range": "± 2160823715",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 18251577,
            "range": "± 596139",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 157111833,
            "range": "± 6782266",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 604458449,
            "range": "± 75035928",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 19096730,
            "range": "± 1949144",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 155578343,
            "range": "± 60434706",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 631261551,
            "range": "± 66372703",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 6707771,
            "range": "± 638973",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "oetyng@gmail.com",
            "name": "oetyng",
            "username": "oetyng"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "ea490ddf749ac9e0c7962c3c21c053663e6b6ee7",
          "message": "chore(naming): reflect the semantics not the type\nThe type is named Kind but the semantics of it is Auth.\nOften we mindlessly name things after the type names instead of what they\nrepresent in the domain.\nBREAKING CHANGE: fields of public msg renamed",
          "timestamp": "2022-07-25T11:08:14+02:00",
          "tree_id": "425f89cfeb16621d587c9bd0201f396016813a34",
          "url": "https://github.com/maidsafe/safe_network/commit/ea490ddf749ac9e0c7962c3c21c053663e6b6ee7"
        },
        "date": 1658742705345,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 656795720,
            "range": "± 39197082",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 1163815397,
            "range": "± 342279411",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 1015793696,
            "range": "± 229399743",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 299030512,
            "range": "± 5321861",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 326156691,
            "range": "± 3625846",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 1084000110,
            "range": "± 224555879",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 182436494,
            "range": "± 30614760",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 1812866545,
            "range": "± 325997643",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 10131496186,
            "range": "± 2141781248",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 210897277,
            "range": "± 95270582",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 1407225002,
            "range": "± 413950177",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 3076596315,
            "range": "± 255624585",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 15069596,
            "range": "± 242965",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 126867929,
            "range": "± 35674181",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 435784040,
            "range": "± 31394426",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 16148094,
            "range": "± 2750463",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 129963639,
            "range": "± 5950558",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 494175435,
            "range": "± 31550484",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 5435423,
            "range": "± 174978",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joshuef@gmail.com",
            "name": "Josh Wilson",
            "username": "joshuef"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "5a83a4ff08b7a3c2be0e3532589d209e72367d92",
          "message": "fix: only do connectivity test for failed sends via dysfunction\n\nThis means we dont message around for every failed send (it may just be\ndue to _us_ cleaning up comms).\n\nInstead we tie this check to a node we see as dysfunctional and then ask\nall elders to check it",
          "timestamp": "2022-07-25T15:00:16+02:00",
          "tree_id": "c951d3bb6b1ebb012c09927e9fa5f4ae9d995dac",
          "url": "https://github.com/maidsafe/safe_network/commit/5a83a4ff08b7a3c2be0e3532589d209e72367d92"
        },
        "date": 1658757885301,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 638419401,
            "range": "± 14338644",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 708404012,
            "range": "± 1650647094",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 992966768,
            "range": "± 2092317404",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 1979352602,
            "range": "± 4919153900",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 10207456653,
            "range": "± 2701569",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 10379504964,
            "range": "± 6963047",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 9740780,
            "range": "± 9334863",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 1075183474,
            "range": "± 303873510",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 6596330858,
            "range": "± 910430000",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 243109048,
            "range": "± 97299021",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 1979896364,
            "range": "± 644893175",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 3966550140,
            "range": "± 2143936547",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 16635142,
            "range": "± 290880",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 133275005,
            "range": "± 2684799",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 484033540,
            "range": "± 37190335",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 16732614,
            "range": "± 939805",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 133737100,
            "range": "± 5527810",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 500207088,
            "range": "± 30259418",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 6201796,
            "range": "± 182659",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joshuef@gmail.com",
            "name": "Josh Wilson",
            "username": "joshuef"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "9c855c187465a0594cf18fb359a082742593a4d4",
          "message": "chore: log parent job id during processing",
          "timestamp": "2022-07-25T16:32:19+02:00",
          "tree_id": "706799d09dbe04c0ac9d05db0c99a297e46b2f6f",
          "url": "https://github.com/maidsafe/safe_network/commit/9c855c187465a0594cf18fb359a082742593a4d4"
        },
        "date": 1658762274166,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 648084668,
            "range": "± 26860066",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 690466571,
            "range": "± 469151535",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 1181776727,
            "range": "± 186588077",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 395075091,
            "range": "± 37111384",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 395764343,
            "range": "± 32120764",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 1095695021,
            "range": "± 60056979",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 13311806,
            "range": "± 9031136",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 1428813500,
            "range": "± 460924205",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 6446500893,
            "range": "± 1212552769",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 250709415,
            "range": "± 105612814",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 2663209046,
            "range": "± 373096191",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 7454305140,
            "range": "± 3132911761",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 13960913,
            "range": "± 597337",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 125060222,
            "range": "± 5354508",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 433841512,
            "range": "± 29098752",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 14735668,
            "range": "± 503484",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 124325160,
            "range": "± 22119613",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 466071872,
            "range": "± 27794884",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 5223035,
            "range": "± 131229",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "qi.ma@maidsafe.net",
            "name": "qima",
            "username": "maqi"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "10ff5b60776f49a606a5b62ae52470709abcf954",
          "message": "chore: enale client log during API tests",
          "timestamp": "2022-07-26T09:38:18+02:00",
          "tree_id": "8ff977271690aa6ad7e2a04119b0968de7ada2db",
          "url": "https://github.com/maidsafe/safe_network/commit/10ff5b60776f49a606a5b62ae52470709abcf954"
        },
        "date": 1658825110168,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 640896872,
            "range": "± 147456042",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 706344986,
            "range": "± 28981577",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 1215824326,
            "range": "± 236899388",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 396225793,
            "range": "± 45653620",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 395776342,
            "range": "± 26743840",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 1101228877,
            "range": "± 39851469",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 17755981,
            "range": "± 26077883",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 2006540049,
            "range": "± 746707598",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 9982809758,
            "range": "± 2335573145",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 236578393,
            "range": "± 219405682",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 3929676583,
            "range": "± 991086010",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 6372286606,
            "range": "± 2955467286",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 14078443,
            "range": "± 869763",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 117990593,
            "range": "± 63684295",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 610249890,
            "range": "± 296345705",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 14063314,
            "range": "± 2074493",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 120171995,
            "range": "± 9451921",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 726006678,
            "range": "± 207331008",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 5448826,
            "range": "± 294081",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joshuef@gmail.com",
            "name": "Josh Wilson",
            "username": "joshuef"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "83a3a98a6972c9a1824e41cb87325b037b65938c",
          "message": "chore: make Sign of SystemMsgs higher prio as this could hold them back before sending",
          "timestamp": "2022-07-26T10:36:28+02:00",
          "tree_id": "13465c432dfe9a7a857b7bcf4d36fbe46b9ad7c6",
          "url": "https://github.com/maidsafe/safe_network/commit/83a3a98a6972c9a1824e41cb87325b037b65938c"
        },
        "date": 1658830922605,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 669324497,
            "range": "± 46709331",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 981039334,
            "range": "± 306957198",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 971020833,
            "range": "± 10133153",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 396455686,
            "range": "± 65391627",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 10203550008,
            "range": "± 3060235008",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 10367726472,
            "range": "± 7247155",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 9232739,
            "range": "± 10505231",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 962583197,
            "range": "± 419312080",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 6153348176,
            "range": "± 1831088057",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 221167041,
            "range": "± 83053982",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 2604461502,
            "range": "± 541243442",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 2733088897,
            "range": "± 1178429464",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 15942119,
            "range": "± 508774",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 125601744,
            "range": "± 14985471",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 431274936,
            "range": "± 26000559",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 14513907,
            "range": "± 291143",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 129241167,
            "range": "± 8646598",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 469183000,
            "range": "± 28993870",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 5311365,
            "range": "± 156636",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joshuef@gmail.com",
            "name": "Josh Wilson",
            "username": "joshuef"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "e9b97c72b860053285ba866b098937f6b25d99bf",
          "message": "Revert \"feat: make traceroute default for now\"\n\nThis reverts commit 175d1b909dff8c6729ac7f156ce1d0d22be8cc12.",
          "timestamp": "2022-07-27T15:53:39+02:00",
          "tree_id": "6d3e98ecf52536811c01a8183b2511b9b402ddb6",
          "url": "https://github.com/maidsafe/safe_network/commit/e9b97c72b860053285ba866b098937f6b25d99bf"
        },
        "date": 1658932911667,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 636234007,
            "range": "± 65543516",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 713801992,
            "range": "± 1186567027",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 1011544691,
            "range": "± 318975370",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 393293945,
            "range": "± 65475703",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 391208494,
            "range": "± 76203961",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 1095384147,
            "range": "± 86945666",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 10941729,
            "range": "± 2781217",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 1792187868,
            "range": "± 376282869",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 6872486820,
            "range": "± 1334974357",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 143489739,
            "range": "± 55160755",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 1846507840,
            "range": "± 709312340",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 5133342229,
            "range": "± 847677792",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 14040516,
            "range": "± 331550",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 123179190,
            "range": "± 12669765",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 425742478,
            "range": "± 36712857",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 14190520,
            "range": "± 492785",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 121738348,
            "range": "± 81486082",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 426205416,
            "range": "± 31526751",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 4611785,
            "range": "± 87816",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "702c33b0d78f4a459725ed0c4538819c949978ce",
          "message": "chore: cleanup cache interface let binding\n\nCo-authored-by: davidrusu <davidrusu.me@gmail.com>",
          "timestamp": "2022-07-28T09:56:12+02:00",
          "tree_id": "39e85c7af2f500feeb05873099916dee19c9e3d2",
          "url": "https://github.com/maidsafe/safe_network/commit/702c33b0d78f4a459725ed0c4538819c949978ce"
        },
        "date": 1658999091143,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 647339884,
            "range": "± 69269975",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 722538068,
            "range": "± 35602029",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 1228362396,
            "range": "± 130598849",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 397409559,
            "range": "± 31272001",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 394971157,
            "range": "± 110024976",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 1104965238,
            "range": "± 80245743",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 18435159,
            "range": "± 5079831",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 1826353723,
            "range": "± 231621216",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 8173465993,
            "range": "± 1214628411",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 359951579,
            "range": "± 127199830",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 2498489615,
            "range": "± 762751736",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 6884528326,
            "range": "± 951756831",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 19675874,
            "range": "± 2993443",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 158824889,
            "range": "± 16999171",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 594764417,
            "range": "± 71799626",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 19171894,
            "range": "± 3295462",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 156378833,
            "range": "± 18014962",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 560408993,
            "range": "± 45157754",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 5467719,
            "range": "± 316068",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "bzeeman@live.nl",
            "name": "Benno Zeeman",
            "username": "b-zee"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "ca32230926e5a435d90694df8fbce1218ea397f0",
          "message": "refactor(client): remove unused storage path",
          "timestamp": "2022-07-28T10:59:04+02:00",
          "tree_id": "07e020c1c5998542580ada65ea2935db3d35e2ad",
          "url": "https://github.com/maidsafe/safe_network/commit/ca32230926e5a435d90694df8fbce1218ea397f0"
        },
        "date": 1659002318354,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 662541901,
            "range": "± 22451666",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 688151981,
            "range": "± 817320756",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 1205987580,
            "range": "± 229525134",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 396086719,
            "range": "± 13988043",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 398447502,
            "range": "± 16997933",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 1095840214,
            "range": "± 57064787",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 10646244,
            "range": "± 4836755",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 1342300138,
            "range": "± 293776215",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 8093032939,
            "range": "± 1026612573",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 213964518,
            "range": "± 85107633",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 1408409714,
            "range": "± 396980052",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 5348068577,
            "range": "± 547342679",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 13073690,
            "range": "± 840543",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 118384918,
            "range": "± 13298902",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 452601702,
            "range": "± 39384338",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 13223469,
            "range": "± 429515",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 118015116,
            "range": "± 9320226",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 419982031,
            "range": "± 28036429",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 5040953,
            "range": "± 174267",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joshuef@gmail.com",
            "name": "Josh Wilson",
            "username": "joshuef"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "22f4f1512da8f9a2245addf379477823364edd6c",
          "message": "fix: create query cmd for healthcheck",
          "timestamp": "2022-07-28T16:00:46+02:00",
          "tree_id": "c0c623b5e330f3601a8cc010250d1a8f0ee38481",
          "url": "https://github.com/maidsafe/safe_network/commit/22f4f1512da8f9a2245addf379477823364edd6c"
        },
        "date": 1659019525892,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 631237119,
            "range": "± 791641115",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 695517654,
            "range": "± 58084269",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 1210792771,
            "range": "± 394364680",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 394864823,
            "range": "± 56245898",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 395739267,
            "range": "± 73790989",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 1108955611,
            "range": "± 59481985",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 8533828,
            "range": "± 4507800",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 799361197,
            "range": "± 172850448",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 7509469962,
            "range": "± 1106456112",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 129467508,
            "range": "± 98533970",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 1070918497,
            "range": "± 244556507",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 4273844651,
            "range": "± 1232430502",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 11858195,
            "range": "± 267032",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 99463449,
            "range": "± 9022586",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 365958734,
            "range": "± 32621307",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 11855134,
            "range": "± 159870",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 100243929,
            "range": "± 4552625",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 370710574,
            "range": "± 26059034",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 4226078,
            "range": "± 77272",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "d0a5f771f40e575dfebb326393202c3249ecca5b",
          "message": "Merge #1419\n\n1419: chore(ci): increase node startup interval to 5s r=oetyng a=oetyng\n\nThis not only removes the ci join issues, but also the fairly common other test errors (not enough spendproof shares, failed to obtain response, not enough chunks etc - seems this could be due to ghost nodes).\n\nWith this we do not need to override our ci all the time due to these tests failing.\n\nAlso it removes the risk of hiding new bugs introduced, behind what we assume were those join related errors.\n\n***\n\nInterval could probably be lowered some. But that'd optimally be done while working on fixing the underlying problem.\n\nGoal for that would be to not have any join-rate that is capable of throwing off the network like that.\n\nCo-authored-by: oetyng <oetyng@gmail.com>",
          "timestamp": "2022-07-30T16:50:33Z",
          "tree_id": "63a6f5e93354861acc9c786a34a99a5b2711c364",
          "url": "https://github.com/maidsafe/safe_network/commit/d0a5f771f40e575dfebb326393202c3249ecca5b"
        },
        "date": 1659207055069,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 657379789,
            "range": "± 15476814",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 694549180,
            "range": "± 14600745",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 1235627231,
            "range": "± 162954905",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 398496104,
            "range": "± 27706683",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 397850636,
            "range": "± 31703464",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 1102028254,
            "range": "± 51775082",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 12706394,
            "range": "± 4445301",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 1621863713,
            "range": "± 592913118",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 7203706143,
            "range": "± 1186219783",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 157973018,
            "range": "± 32004276",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 1365501793,
            "range": "± 367661933",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 5513370013,
            "range": "± 538035962",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 11108728,
            "range": "± 333288",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 102447673,
            "range": "± 50110230",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 388187029,
            "range": "± 28760553",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 11110160,
            "range": "± 441440",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 105184860,
            "range": "± 9921357",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 358064860,
            "range": "± 30196337",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 4966569,
            "range": "± 157670",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joshuef@gmail.com",
            "name": "Josh Wilson",
            "username": "joshuef"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "c165edc21f4105b38fcaa84252f4dd8134d0e4c3",
          "message": "ci: ensure we prebuild in the crate dir before running the tests",
          "timestamp": "2022-08-01T08:03:38+02:00",
          "tree_id": "426db15d68412137784cf2166cc435ee2ff392e9",
          "url": "https://github.com/maidsafe/safe_network/commit/c165edc21f4105b38fcaa84252f4dd8134d0e4c3"
        },
        "date": 1659337497885,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 645947135,
            "range": "± 32720010",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 674369288,
            "range": "± 18860133",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 1189515508,
            "range": "± 653969003",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 399766336,
            "range": "± 9114186",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 398754138,
            "range": "± 11841445",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 1110403522,
            "range": "± 40205746",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 10283118,
            "range": "± 19072069",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 1500346020,
            "range": "± 360463983",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 7591789737,
            "range": "± 1460755833",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 340912059,
            "range": "± 132271161",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 1421959664,
            "range": "± 657964698",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 5041607347,
            "range": "± 633451072",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 12567799,
            "range": "± 445425",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 111914397,
            "range": "± 9950912",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 446825082,
            "range": "± 22552746",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 12680407,
            "range": "± 511897",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 111159456,
            "range": "± 18881759",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 427152251,
            "range": "± 32977625",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 5030838,
            "range": "± 122750",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "oetyng@gmail.com",
            "name": "oetyng",
            "username": "oetyng"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "080f9ef83005ebda9e1c96b228f3d5096fd79b81",
          "message": "chore: delete commented out tests",
          "timestamp": "2022-08-01T13:06:33+02:00",
          "tree_id": "bb3c497eea802143328e9a812de16b72885c7608",
          "url": "https://github.com/maidsafe/safe_network/commit/080f9ef83005ebda9e1c96b228f3d5096fd79b81"
        },
        "date": 1659356243742,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 637635837,
            "range": "± 36781008",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 680186225,
            "range": "± 28298280",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 10806393027,
            "range": "± 3001448555",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10191325594,
            "range": "± 6415949",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 10208342112,
            "range": "± 5187914",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 10363757196,
            "range": "± 11584179",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 12807285,
            "range": "± 4479813",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 1492868130,
            "range": "± 469026375",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 7107886984,
            "range": "± 1210160055",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 315032224,
            "range": "± 112334951",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 1437017418,
            "range": "± 447570143",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 5228808496,
            "range": "± 686072648",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 12744932,
            "range": "± 681920",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 118050084,
            "range": "± 10290371",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 438591728,
            "range": "± 499459062",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 12764436,
            "range": "± 944284",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 118834771,
            "range": "± 2929162",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 423327181,
            "range": "± 28164228",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 5263333,
            "range": "± 122066",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "RolandSherwin@protonmail.com",
            "name": "RolandSherwin",
            "username": "RolandSherwin"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "9d3bf7e79684b69f8eb8f03978988aee5a8b2c00",
          "message": "chore(CI): cancel old CI run on new commit\n\nStops the old CI run on force push/ new commit.\nEg: any push to `my_branch` will create group with name,\n\"PR Check-my_branch\" and subsequent updates to the branch\nwill stop any running CI for the same group.",
          "timestamp": "2022-08-01T13:40:59+02:00",
          "tree_id": "14ca877e57449a49fe734a0143099a436f9b7431",
          "url": "https://github.com/maidsafe/safe_network/commit/9d3bf7e79684b69f8eb8f03978988aee5a8b2c00"
        },
        "date": 1659357330707,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 657355893,
            "range": "± 24366478",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 697807783,
            "range": "± 80581830",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 1228867115,
            "range": "± 188697120",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 398987152,
            "range": "± 21128110",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 397846559,
            "range": "± 25333081",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 1099125239,
            "range": "± 22478528",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 8849509,
            "range": "± 2983831",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 1340972552,
            "range": "± 448777321",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 7140805230,
            "range": "± 1220338113",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 136974929,
            "range": "± 153404137",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 2762821851,
            "range": "± 805952514",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 5024301926,
            "range": "± 1826549971",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 11906064,
            "range": "± 460040",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 98848313,
            "range": "± 3435353",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 400580206,
            "range": "± 38305691",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 12435656,
            "range": "± 122304",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 100910866,
            "range": "± 7974171",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 391602981,
            "range": "± 30688364",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 4398521,
            "range": "± 176677",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "qi.ma@maidsafe.net",
            "name": "qima",
            "username": "maqi"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "8e1b834b14fe7145b1b3583637c02bfa6287ea94",
          "message": "chore: reduce the testing threads for the CLI tests",
          "timestamp": "2022-08-01T13:40:31+02:00",
          "tree_id": "e3ee603a9017093fe37dcb9eef699ca237dffa50",
          "url": "https://github.com/maidsafe/safe_network/commit/8e1b834b14fe7145b1b3583637c02bfa6287ea94"
        },
        "date": 1659357781720,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 656567939,
            "range": "± 19875941",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 690083438,
            "range": "± 22687080",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 1249276793,
            "range": "± 51034319",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 397261433,
            "range": "± 25665964",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 403994944,
            "range": "± 59594665",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 1091866323,
            "range": "± 63960693",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 12280335,
            "range": "± 3500861",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 1178210685,
            "range": "± 426494344",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 6944860465,
            "range": "± 1404132816",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 293695130,
            "range": "± 122926074",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 2542368844,
            "range": "± 623451585",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 8821771876,
            "range": "± 2225320277",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 13521741,
            "range": "± 791976",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 116076228,
            "range": "± 8328674",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 441364915,
            "range": "± 43866586",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 14659825,
            "range": "± 734849",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 120939272,
            "range": "± 12553695",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 420707380,
            "range": "± 51771837",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 5402787,
            "range": "± 104168",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "26634292+bors[bot]@users.noreply.github.com",
            "name": "bors[bot]",
            "username": "bors[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": false,
          "id": "949ee111717c8f07487f3f4db6fbc0043583916d",
          "message": "Merge #1427\n\n1427: chore(CI): cancel old CI run on new commit r=joshuef a=RolandSherwin\n\nStops the old CI run on force push/ new commit.\r\n\r\nEg: any push to `my_branch` will create group with name,\r\n\"PR Check-my_branch\" and subsequent updates to the branch\r\nwill stop any running CI for the same group.\r\n\r\n\n\nCo-authored-by: RolandSherwin <RolandSherwin@protonmail.com>",
          "timestamp": "2022-08-01T12:32:22Z",
          "tree_id": "14ca877e57449a49fe734a0143099a436f9b7431",
          "url": "https://github.com/maidsafe/safe_network/commit/949ee111717c8f07487f3f4db6fbc0043583916d"
        },
        "date": 1659364861410,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 629063592,
            "range": "± 137581886",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 688142861,
            "range": "± 340109171",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 1212205815,
            "range": "± 126418773",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 397686389,
            "range": "± 31510355",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 395633430,
            "range": "± 36882466",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 10349833911,
            "range": "± 3629718829",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 8823404,
            "range": "± 20426274",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 753813651,
            "range": "± 236350034",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 6961656133,
            "range": "± 1346465144",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 206187160,
            "range": "± 95811860",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 2199157543,
            "range": "± 720963459",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 5083671770,
            "range": "± 2285030995",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 11036791,
            "range": "± 215252",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 98353777,
            "range": "± 5820231",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 377720263,
            "range": "± 27659149",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 10985369,
            "range": "± 288479",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 97396146,
            "range": "± 17066400",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 337826238,
            "range": "± 33939193",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 4192338,
            "range": "± 60772",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "oetyng@gmail.com",
            "name": "oetyng",
            "username": "oetyng"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "db4f4d07b155d732ad76d263563d81b5fee535f7",
          "message": "refactor(messaging): remove more unused code\nMore reuse of methods to replace duplication of code.\nDeprecates delivery group, since it is no longer used.\nAlso, `DstLocation` and `SrcLocation` are removed.\nBREAKING CHANGE: WireMsg public type is changed.",
          "timestamp": "2022-08-01T16:55:51+02:00",
          "tree_id": "441638449092d934beeba1c9be5d1e539a0f5e1a",
          "url": "https://github.com/maidsafe/safe_network/commit/db4f4d07b155d732ad76d263563d81b5fee535f7"
        },
        "date": 1659368660885,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 597822273,
            "range": "± 20308533",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 643213186,
            "range": "± 30298250",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 1241142531,
            "range": "± 44582705",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 399635903,
            "range": "± 16823791",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 397971298,
            "range": "± 19932767",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 1098848105,
            "range": "± 46389537",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 10424938,
            "range": "± 4088991",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 1292735256,
            "range": "± 483197115",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 6699457453,
            "range": "± 1050585629",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 216751860,
            "range": "± 73831487",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 2161070772,
            "range": "± 680131404",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 6188446564,
            "range": "± 1445286072",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 12948716,
            "range": "± 439099",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 110641752,
            "range": "± 3781723",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 407374035,
            "range": "± 34289435",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 12812674,
            "range": "± 352390",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 110456955,
            "range": "± 5130070",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 403834055,
            "range": "± 38844168",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 5202642,
            "range": "± 117707",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "bzeeman@live.nl",
            "name": "Benno Zeeman",
            "username": "b-zee"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "db7dcdc7968d1d7e946274650d5a0c48719b4955",
          "message": "chore!: remove providing path to qp2p cfg\n\nThis configuration seems never to be provided or stored anyway. It looks\nlike some code was also taking this parameter to be the client config,\nnot the qp2p config, which is a source of confusion.\n\nBREAKING CHANGE: One less argument to pass to `ClientConfig::new`",
          "timestamp": "2022-08-01T16:56:24+02:00",
          "tree_id": "317870c09adb0db0c9013edc7c536e6124570946",
          "url": "https://github.com/maidsafe/safe_network/commit/db7dcdc7968d1d7e946274650d5a0c48719b4955"
        },
        "date": 1659368716640,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 598729305,
            "range": "± 186849117",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 651444342,
            "range": "± 201286072",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 1232486077,
            "range": "± 123287142",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 396794920,
            "range": "± 19865067",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 400516648,
            "range": "± 47669298",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 1092332349,
            "range": "± 3831422297",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 8815488,
            "range": "± 9222424",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 460758312,
            "range": "± 53297605",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 6218536054,
            "range": "± 1054348760",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 263001953,
            "range": "± 83326145",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 2421275733,
            "range": "± 554529840",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 5032298648,
            "range": "± 1119781122",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 11208270,
            "range": "± 234105",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 94980047,
            "range": "± 197333911",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 423883010,
            "range": "± 50456853",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 11825660,
            "range": "± 361259",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 95029766,
            "range": "± 18717324",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 357896171,
            "range": "± 21830070",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 4070762,
            "range": "± 116992",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "oetyng@gmail.com",
            "name": "oetyng",
            "username": "oetyng"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "0f07efd9ef0b75de79f27772566b013bc886bcc8",
          "message": "refactor(joinrequest): remove optional field",
          "timestamp": "2022-08-01T19:36:44+02:00",
          "tree_id": "4793bb1c281fe6f958862e2c6794baa7dc82c329",
          "url": "https://github.com/maidsafe/safe_network/commit/0f07efd9ef0b75de79f27772566b013bc886bcc8"
        },
        "date": 1659378077560,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 590445113,
            "range": "± 24940113",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 653203176,
            "range": "± 618744327",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 1231724045,
            "range": "± 202338651",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 397693081,
            "range": "± 427082654",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 10199907051,
            "range": "± 1936228",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 10345183315,
            "range": "± 7210260",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 8350802,
            "range": "± 6395993",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 1264471058,
            "range": "± 514255623",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 6262836287,
            "range": "± 1794081146",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 235995374,
            "range": "± 116190277",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 2607991967,
            "range": "± 341423800",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 8497548294,
            "range": "± 2057368732",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 11186576,
            "range": "± 198302",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 97830739,
            "range": "± 37709548",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 366135696,
            "range": "± 31961212",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 11036851,
            "range": "± 173664",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 101785384,
            "range": "± 42775217",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 388565954,
            "range": "± 27883282",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 4236415,
            "range": "± 71809",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "oetyng@gmail.com",
            "name": "oetyng",
            "username": "oetyng"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "a0c89ff0e451d2e5dd13fc29635075097f2c7b94",
          "message": "refactor: do not require node write lock on query\nThis creates the `AddToPendingQieries` cmd, which adds asyncly to the\nlist.\nAlso cleans up the `read_data_from_adults` fn a bit.",
          "timestamp": "2022-08-01T19:38:15+02:00",
          "tree_id": "69178d11636942314ebdd7279e4f4440f1b88423",
          "url": "https://github.com/maidsafe/safe_network/commit/a0c89ff0e451d2e5dd13fc29635075097f2c7b94"
        },
        "date": 1659379313480,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 628659470,
            "range": "± 766034206",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 658364135,
            "range": "± 171084435",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 1268531862,
            "range": "± 667203920",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 397981177,
            "range": "± 36085694",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 398299339,
            "range": "± 39217312",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 1695493401,
            "range": "± 4703252589",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 11748225,
            "range": "± 5012797",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 1565738373,
            "range": "± 508379094",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 7504843606,
            "range": "± 1059936851",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 144686742,
            "range": "± 55681846",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 1486590803,
            "range": "± 263535815",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 5626766722,
            "range": "± 644758294",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 13261607,
            "range": "± 447568",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 112198501,
            "range": "± 12429928",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 402498630,
            "range": "± 34221237",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 13494036,
            "range": "± 360702",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 112408910,
            "range": "± 4813318",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 418678074,
            "range": "± 30322048",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 5242215,
            "range": "± 415485",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "oetyng@gmail.com",
            "name": "oetyng",
            "username": "oetyng"
          },
          "committer": {
            "email": "davidrusu.me@gmail.com",
            "name": "davidrusu",
            "username": "davidrusu"
          },
          "distinct": true,
          "id": "cebf37ea8ef44c51ce84646a83a5d1e6dcab3e7a",
          "message": "fix(file_store): use correct data address type\nThe type including `SafeKey` had been incorrectly used (since it is not\na network side concept), which caused a lot of `Result` return values\nbloating the call tree unecessarily.",
          "timestamp": "2022-08-01T18:22:38-04:00",
          "tree_id": "032e5dbcf2e315c9c8ebf3e94adb12dabc5a5b47",
          "url": "https://github.com/maidsafe/safe_network/commit/cebf37ea8ef44c51ce84646a83a5d1e6dcab3e7a"
        },
        "date": 1659396057887,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 605410779,
            "range": "± 11248845",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 663681772,
            "range": "± 953108136",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 920179373,
            "range": "± 3363318600",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 463399939,
            "range": "± 4680030025",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 10205219730,
            "range": "± 7295243",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 10352963643,
            "range": "± 11902624",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 9704081,
            "range": "± 5372038",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 1256356737,
            "range": "± 307245292",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 6318281722,
            "range": "± 922244183",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 200168407,
            "range": "± 64155944",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 1336946772,
            "range": "± 322912255",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 5276876018,
            "range": "± 989323968",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 11127506,
            "range": "± 595458",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 99850310,
            "range": "± 3886434",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 345239840,
            "range": "± 25221785",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 11259262,
            "range": "± 264596",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 101340675,
            "range": "± 7494863",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 340438201,
            "range": "± 27964400",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 4429340,
            "range": "± 84584",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "bzeeman@live.nl",
            "name": "Benno Zeeman",
            "username": "b-zee"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "14ea6c7f4bbaee9c2ac4a30fba938ef2de2f77e5",
          "message": "refactor(client): use builder to instantiate",
          "timestamp": "2022-08-03T16:02:45+02:00",
          "tree_id": "a479710ce268ba181bd506e913d65f4292597ec9",
          "url": "https://github.com/maidsafe/safe_network/commit/14ea6c7f4bbaee9c2ac4a30fba938ef2de2f77e5"
        },
        "date": 1659540796583,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 610601313,
            "range": "± 16430460",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 673033673,
            "range": "± 191354798",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 1204439916,
            "range": "± 859450558",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10189294234,
            "range": "± 4143090913",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 10213340714,
            "range": "± 4092682",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 10398204694,
            "range": "± 83229684",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 11910641,
            "range": "± 8173634",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 1445859431,
            "range": "± 374389780",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 7030545785,
            "range": "± 1534510915",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 274315926,
            "range": "± 90855355",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 1738955108,
            "range": "± 722349025",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 4910870385,
            "range": "± 383362244",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 15487265,
            "range": "± 484883",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 137704314,
            "range": "± 45942693",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 672308088,
            "range": "± 95106014",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 15934969,
            "range": "± 508585",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 135578194,
            "range": "± 13602671",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 516568150,
            "range": "± 30802815",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 5933113,
            "range": "± 370649",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "RolandSherwin@protonmail.com",
            "name": "RolandSherwin",
            "username": "RolandSherwin"
          },
          "committer": {
            "email": "davidrusu.me@gmail.com",
            "name": "davidrusu",
            "username": "davidrusu"
          },
          "distinct": true,
          "id": "60ec545c4ff2d56c4b92ecbf3b710088a8339450",
          "message": "chore(cli): make writes atomic",
          "timestamp": "2022-08-03T11:24:30-04:00",
          "tree_id": "f44ef7f4a439c6771bc29ac306a960654bcc4a42",
          "url": "https://github.com/maidsafe/safe_network/commit/60ec545c4ff2d56c4b92ecbf3b710088a8339450"
        },
        "date": 1659543054678,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 613006030,
            "range": "± 17077462",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 647955896,
            "range": "± 291002772",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 1233572412,
            "range": "± 198642092",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 399497655,
            "range": "± 1233265230",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 10205207585,
            "range": "± 6055501",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 10360384200,
            "range": "± 7259958",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 8963819,
            "range": "± 4340729",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 413252189,
            "range": "± 128663431",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 6309389328,
            "range": "± 1192094392",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 112092300,
            "range": "± 55234716",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 1131547970,
            "range": "± 192680116",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 4602424581,
            "range": "± 396522663",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 12027460,
            "range": "± 258546",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 100324672,
            "range": "± 45023120",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 431229005,
            "range": "± 52246760",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 12006166,
            "range": "± 269680",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 98300602,
            "range": "± 4757141",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 348946536,
            "range": "± 32385426",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 4560564,
            "range": "± 175775",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joshuef@gmail.com",
            "name": "Josh Wilson",
            "username": "joshuef"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "72db95a092ea33f29e77b6101b16e219fadd47ab",
          "message": "chore: refactor CmdCtrl, naming and remove retries",
          "timestamp": "2022-08-04T17:13:20+02:00",
          "tree_id": "e5f58a3bdd0afe67b460e2c38b2f3fd819247283",
          "url": "https://github.com/maidsafe/safe_network/commit/72db95a092ea33f29e77b6101b16e219fadd47ab"
        },
        "date": 1659629538394,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 2929224704,
            "range": "± 4945382164",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 10536542046,
            "range": "± 104908744",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 10735576186,
            "range": "± 1204099411",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10183902980,
            "range": "± 14209393",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 10201421948,
            "range": "± 2938113",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 10355787427,
            "range": "± 12434009",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 8829444,
            "range": "± 9827133",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 1406603365,
            "range": "± 383254439",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 6356896809,
            "range": "± 1233765093",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 172218974,
            "range": "± 106667251",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 2521347079,
            "range": "± 561780025",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 3953982756,
            "range": "± 1207660838",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 10800163,
            "range": "± 182070",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 98171120,
            "range": "± 19725409",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 422332293,
            "range": "± 60753008",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 10979541,
            "range": "± 223155",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 103674467,
            "range": "± 15772132",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 386861355,
            "range": "± 22961804",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 4352989,
            "range": "± 111991",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "RolandSherwin@protonmail.com",
            "name": "RolandSherwin",
            "username": "RolandSherwin"
          },
          "committer": {
            "email": "davidrusu.me@gmail.com",
            "name": "davidrusu",
            "username": "davidrusu"
          },
          "distinct": true,
          "id": "848dba48e5959d0b9cfe182fde2f12ede71ba9c2",
          "message": "chore(prefix_map): use matches! macros, minor refactoring",
          "timestamp": "2022-08-04T15:18:57-04:00",
          "tree_id": "1e7f59d9ee10d9be94921233b6620c7fe12e5434",
          "url": "https://github.com/maidsafe/safe_network/commit/848dba48e5959d0b9cfe182fde2f12ede71ba9c2"
        },
        "date": 1659645400075,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 752896091,
            "range": "± 226315239",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 767672725,
            "range": "± 263547463",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 1196442296,
            "range": "± 227989540",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 398351660,
            "range": "± 30852052",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 10024162117,
            "range": "± 5015629644",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 10335807120,
            "range": "± 7958215",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 8312523,
            "range": "± 2578484",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 1338055463,
            "range": "± 388388480",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 6494954635,
            "range": "± 1462036496",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 130383121,
            "range": "± 38559944",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 1129390357,
            "range": "± 239800971",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 4205228547,
            "range": "± 553652007",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 10786112,
            "range": "± 131679",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 103596612,
            "range": "± 57790006",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 386212118,
            "range": "± 41776603",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 10993130,
            "range": "± 276275",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 100922508,
            "range": "± 20931105",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 382164619,
            "range": "± 26770904",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 4333731,
            "range": "± 126988",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "oetyng@gmail.com",
            "name": "oetyng",
            "username": "oetyng"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "8242f2f1035b1c0718e53954951badffa30f3393",
          "message": "chore(style): organise usings, cleanup\n- Removes some boilerplate, using fn of `Cmd` to instantiate a send cmd.\n- Housekeeping, continuing to minimize bloat of usings, by colocating\nthem.\n- Housekeeping, continuing keeping positions of usings in a file\naccording to a system, from closest (self) on top, down to furthest\naway (3rd part).",
          "timestamp": "2022-08-05T16:38:50+02:00",
          "tree_id": "1e30e67e37c8b822f03ca8c5921dd48a0d72ffe6",
          "url": "https://github.com/maidsafe/safe_network/commit/8242f2f1035b1c0718e53954951badffa30f3393"
        },
        "date": 1659713949581,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 598845226,
            "range": "± 14684109",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 655648745,
            "range": "± 30879682",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 1214451275,
            "range": "± 140497287",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 399665615,
            "range": "± 19729749",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 396635314,
            "range": "± 12360500",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 1107873044,
            "range": "± 21902352",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 10841392,
            "range": "± 5362815",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 1658821097,
            "range": "± 462514219",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 7420524239,
            "range": "± 1470859304",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 365933592,
            "range": "± 135352132",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 1617038182,
            "range": "± 992654806",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 4303010884,
            "range": "± 315566136",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 13055358,
            "range": "± 650582",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 117894108,
            "range": "± 6637077",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 453737959,
            "range": "± 71832818",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 12751419,
            "range": "± 472185",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 115007351,
            "range": "± 66009984",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 448471303,
            "range": "± 39822441",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 5017306,
            "range": "± 151507",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "chriso83@protonmail.com",
            "name": "Chris O'Neil",
            "username": "jacderida"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "4a439431cd5b174c523e78503060516b1943185d",
          "message": "ci(nightly): upload logs for failed run\n\nFor a failed run, before the testnet is killed, the node logs are retrieved and uploaded to S3.\nAlong with the logs, the prefix map, the genesis key and the genesis DBC will also be uploaded.\n\nThis is all to facilitate debugging test failures.",
          "timestamp": "2022-08-09T08:16:57+02:00",
          "tree_id": "88a9a4576f4b4da29131c96d29310dd4870d19f7",
          "url": "https://github.com/maidsafe/safe_network/commit/4a439431cd5b174c523e78503060516b1943185d"
        },
        "date": 1660028482840,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 476559841,
            "range": "± 61453850",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 470011218,
            "range": "± 388365777",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 962773189,
            "range": "± 53294310",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 331338995,
            "range": "± 9913159",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 340751397,
            "range": "± 5799959",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 791684310,
            "range": "± 38508011",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 8679742,
            "range": "± 2837263",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 486457591,
            "range": "± 97714140",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 6663730601,
            "range": "± 1599429419",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 241664884,
            "range": "± 80504762",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 2278230268,
            "range": "± 359443644",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 8598945116,
            "range": "± 2299120523",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 11253106,
            "range": "± 232947",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 95014539,
            "range": "± 4753013",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 336142089,
            "range": "± 37740836",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 11231874,
            "range": "± 155078",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 95106846,
            "range": "± 5080148",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 328128123,
            "range": "± 29581221",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 4237402,
            "range": "± 86147",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "gabrielviganotti@gmail.com",
            "name": "bochaco",
            "username": "bochaco"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "96da1171d0cac240f772e5d6a15c56f63441b4b3",
          "message": "refactor(node)!: nodes to cache their own individual prefix map file on disk",
          "timestamp": "2022-08-09T08:31:32+02:00",
          "tree_id": "4d8e68d8634ffb8f4e50fefa2fcc7447d5898dac",
          "url": "https://github.com/maidsafe/safe_network/commit/96da1171d0cac240f772e5d6a15c56f63441b4b3"
        },
        "date": 1660029296918,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 486831554,
            "range": "± 10286112",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 519528261,
            "range": "± 18945975",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 975268120,
            "range": "± 65784650",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 334217389,
            "range": "± 12639271",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 343217021,
            "range": "± 8758353",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 803481227,
            "range": "± 64423462",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 9582480,
            "range": "± 1584212",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 646788522,
            "range": "± 175994606",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 6091238434,
            "range": "± 1024951821",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 142131248,
            "range": "± 66068223",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 1323620621,
            "range": "± 377145209",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 4642626064,
            "range": "± 410088555",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 11791519,
            "range": "± 240081",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 100386628,
            "range": "± 20966490",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 339305574,
            "range": "± 29103369",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 11723331,
            "range": "± 375582",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 97570320,
            "range": "± 12502402",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 371551283,
            "range": "± 26677407",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 4479847,
            "range": "± 121663",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "RolandSherwin@protonmail.com",
            "name": "RolandSherwin",
            "username": "RolandSherwin"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "6e65ed8e6c5872bd2c49a1ed2837b1fb16523af1",
          "message": "refactor(network_knowledge): remove NetworkKnowledge::chain",
          "timestamp": "2022-08-10T11:42:03+02:00",
          "tree_id": "db4f1f85592c32ef18c8de38419d54ca5216328c",
          "url": "https://github.com/maidsafe/safe_network/commit/6e65ed8e6c5872bd2c49a1ed2837b1fb16523af1"
        },
        "date": 1660127840450,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 490198343,
            "range": "± 9678924",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 496470773,
            "range": "± 42156639",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 897025204,
            "range": "± 94656028",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 331456262,
            "range": "± 9116867",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 342270225,
            "range": "± 6333279",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 802647537,
            "range": "± 37565445",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 10160794,
            "range": "± 1560432",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 1278918426,
            "range": "± 417528349",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 7126283652,
            "range": "± 1953035494",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 115176123,
            "range": "± 65271095",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 1130287803,
            "range": "± 208177934",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 4503802458,
            "range": "± 442168905",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 12008097,
            "range": "± 507395",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 107792314,
            "range": "± 17839768",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 398694773,
            "range": "± 46196953",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 12046387,
            "range": "± 293814",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 105849718,
            "range": "± 4845740",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 397174063,
            "range": "± 26297911",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 24898006,
            "range": "± 538208",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joshuef@gmail.com",
            "name": "Josh Wilson",
            "username": "joshuef"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "ae353f1c892b8285751228a4bde5c9d036544b5d",
          "message": "chore(testnet): small improvements to flamegraph init and cos",
          "timestamp": "2022-08-10T11:40:53+02:00",
          "tree_id": "1e1cdea8cedaedcf2f34eb584ec7c6c21e93bd29",
          "url": "https://github.com/maidsafe/safe_network/commit/ae353f1c892b8285751228a4bde5c9d036544b5d"
        },
        "date": 1660128098007,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 648275231,
            "range": "± 549729253",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 500455021,
            "range": "± 37912930",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 996331063,
            "range": "± 77285846",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 333846941,
            "range": "± 11038428",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 341348853,
            "range": "± 7118913",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 809707271,
            "range": "± 4009121986",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 9116688,
            "range": "± 14187581",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 789967360,
            "range": "± 189468610",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 6341159327,
            "range": "± 1008354316",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 104659464,
            "range": "± 56534732",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 1158069958,
            "range": "± 315503302",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 4574350097,
            "range": "± 687233039",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 11495968,
            "range": "± 221488",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 101488657,
            "range": "± 4660822",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 387390651,
            "range": "± 23242144",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 12463905,
            "range": "± 6808574",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 102158736,
            "range": "± 49781492",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 390261800,
            "range": "± 168440531",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 22629698,
            "range": "± 2048390",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "bzeeman@live.nl",
            "name": "Benno Zeeman",
            "username": "b-zee"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "753443da697a61e49eac977402731c4373e7f4f9",
          "message": "docs: add client builder code example",
          "timestamp": "2022-08-10T15:10:28+02:00",
          "tree_id": "5b0c4ab956c5c0d07f778c46cf4a0fcbe8b02cdf",
          "url": "https://github.com/maidsafe/safe_network/commit/753443da697a61e49eac977402731c4373e7f4f9"
        },
        "date": 1660139669630,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 1062844474,
            "range": "± 890485987",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 522298252,
            "range": "± 19872941",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 960204426,
            "range": "± 78831124",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 332652607,
            "range": "± 12227143",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 337445610,
            "range": "± 9395090",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 799264322,
            "range": "± 22845632",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 8935274,
            "range": "± 9967761",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 1388776900,
            "range": "± 414275090",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 7102520050,
            "range": "± 1734922475",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 123533336,
            "range": "± 55668530",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 1254643280,
            "range": "± 336197265",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 4321003745,
            "range": "± 730037423",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 11009887,
            "range": "± 201229",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 99601432,
            "range": "± 37252381",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 346054912,
            "range": "± 26085104",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 11631705,
            "range": "± 441117",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 99198016,
            "range": "± 6937140",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 385502284,
            "range": "± 27255974",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 22860984,
            "range": "± 253789",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joshuef@gmail.com",
            "name": "Josh Wilson",
            "username": "joshuef"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "a5afec84bc687327df3951eefc6c566c898f2332",
          "message": "fix(node): Add braces around expected age calc; use membership\n\nPreviously we were occasionally seeing _large_ expected ages (246 eg),\nhere we add braces for clarity and hopefully prevent such calculatory oddness.\n\nWe also use membership to know the current section size instead of SAP which\nmay well be outdated.",
          "timestamp": "2022-08-10T15:21:50+02:00",
          "tree_id": "bcaaed2c4b69b0ac82d8b783ffd6b8d9ac4d9413",
          "url": "https://github.com/maidsafe/safe_network/commit/a5afec84bc687327df3951eefc6c566c898f2332"
        },
        "date": 1660140313452,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 476957973,
            "range": "± 82629670",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 523846162,
            "range": "± 18830730",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 979603469,
            "range": "± 77855951",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 328966849,
            "range": "± 10115744",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 342292021,
            "range": "± 4140999",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 806597036,
            "range": "± 64755741",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 9041738,
            "range": "± 1149003",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 1342596920,
            "range": "± 402760270",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 7059554061,
            "range": "± 1674391754",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 255209434,
            "range": "± 102213408",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 968827787,
            "range": "± 176608073",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 4041648934,
            "range": "± 342535435",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 10976749,
            "range": "± 235258",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 98732632,
            "range": "± 12280063",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 341970453,
            "range": "± 24604341",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 10728928,
            "range": "± 247476",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 97622032,
            "range": "± 9484013",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 339347407,
            "range": "± 18166437",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 21541824,
            "range": "± 243045",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joshuef@gmail.com",
            "name": "Josh Wilson",
            "username": "joshuef"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "3cf903367bfcd805ceff2f2508cd2b12eddc3ca5",
          "message": "chore(dysfunction): remove unused severity; refactor weighted score\n\nPrev weighted score related everything to the std_deviation, but this\nhas the effect of nullifying outliers and decreasing the impact of\nweighting.\n\nInstead we opt for a simple \"threshold\" score, above which, we're\ndysfunctional. So the sum of all issues tracked is used, and if\nwe reach above this point, our node is deemed dysfunctional.",
          "timestamp": "2022-08-10T15:50:01+02:00",
          "tree_id": "dd3a1d93f493281d363d025a8c35298eb99ba15f",
          "url": "https://github.com/maidsafe/safe_network/commit/3cf903367bfcd805ceff2f2508cd2b12eddc3ca5"
        },
        "date": 1660142263400,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 454592777,
            "range": "± 184853578",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 1420657472,
            "range": "± 482922142",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 814125700,
            "range": "± 1154380797",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 338090394,
            "range": "± 19361018",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 344295802,
            "range": "± 7482125",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 804926488,
            "range": "± 53255848",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 9004578,
            "range": "± 2560176",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 731117436,
            "range": "± 177475976",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 7124609017,
            "range": "± 1711112629",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 125013686,
            "range": "± 58727925",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 1322288008,
            "range": "± 345126885",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 4631391329,
            "range": "± 463954004",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 12163397,
            "range": "± 281417",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 100687588,
            "range": "± 4611055",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 346301987,
            "range": "± 25370059",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 12218204,
            "range": "± 277109",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 100612797,
            "range": "± 19640886",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 353828760,
            "range": "± 30535987",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 22300144,
            "range": "± 439065",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "qi.ma@maidsafe.net",
            "name": "qima",
            "username": "maqi"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "f531c62b0d82f14b6aa6df4f1f82bcd0ce95b9ce",
          "message": "fix(node): modify the join process timeout mechanism",
          "timestamp": "2022-08-10T18:03:08+02:00",
          "tree_id": "f54656e8951d23c6b7dbb58ebcdfbd326e9da5b7",
          "url": "https://github.com/maidsafe/safe_network/commit/f531c62b0d82f14b6aa6df4f1f82bcd0ce95b9ce"
        },
        "date": 1660149946629,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 485496528,
            "range": "± 11745769",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 525525611,
            "range": "± 10304355",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 979384888,
            "range": "± 77439044",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 332026987,
            "range": "± 4768006",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 340813649,
            "range": "± 4186405",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 782839407,
            "range": "± 58488679",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 9274976,
            "range": "± 3611462",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 1361737725,
            "range": "± 399189928",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 5820055096,
            "range": "± 832956901",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 142320896,
            "range": "± 38777642",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 1238130195,
            "range": "± 309973177",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 4202472050,
            "range": "± 470235091",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 11046577,
            "range": "± 226508",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 106104078,
            "range": "± 10439940",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 384264124,
            "range": "± 47124490",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 11106888,
            "range": "± 329221",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 104515786,
            "range": "± 7973182",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 382038605,
            "range": "± 35446802",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 22112624,
            "range": "± 157871",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "gabrielviganotti@gmail.com",
            "name": "bochaco",
            "username": "bochaco"
          },
          "committer": {
            "email": "gabrielviganotti@gmail.com",
            "name": "bochaco",
            "username": "bochaco"
          },
          "distinct": true,
          "id": "95c33d1ea2040bce4078be96ed8b1c9f2e966b21",
          "message": "test(client): have many-clients test to report the errors found when instantiating clients",
          "timestamp": "2022-08-10T13:08:34-03:00",
          "tree_id": "6722204b2879edd576db2cc158debfddcc89840f",
          "url": "https://github.com/maidsafe/safe_network/commit/95c33d1ea2040bce4078be96ed8b1c9f2e966b21"
        },
        "date": 1660151485062,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 484856320,
            "range": "± 10337691",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 523839895,
            "range": "± 14695421",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 887135649,
            "range": "± 669961378",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 334306859,
            "range": "± 8571632",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 345764440,
            "range": "± 3438970",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 10361570113,
            "range": "± 4019049423",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 9003408,
            "range": "± 887363",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 1521652765,
            "range": "± 298601462",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 5782669737,
            "range": "± 1206609498",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 127258687,
            "range": "± 34707518",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 1274661007,
            "range": "± 268246382",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 4566369279,
            "range": "± 434643413",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 10821231,
            "range": "± 235899",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 97391820,
            "range": "± 17906560",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 401486355,
            "range": "± 34334931",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 11065202,
            "range": "± 286294",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 98569451,
            "range": "± 20053685",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 373815828,
            "range": "± 33090885",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 22258775,
            "range": "± 354242",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joshuef@gmail.com",
            "name": "Josh Wilson",
            "username": "joshuef"
          },
          "committer": {
            "email": "davidrusu.me@gmail.com",
            "name": "davidrusu",
            "username": "davidrusu"
          },
          "distinct": true,
          "id": "6d60525874dc4efeb658433f1f253d54e0cba2d4",
          "message": "chore: remove wiremsg.priority as uneeded",
          "timestamp": "2022-08-10T14:24:51-04:00",
          "tree_id": "51a429a9f26eb3a0994e6fcd3b72a61ff7705336",
          "url": "https://github.com/maidsafe/safe_network/commit/6d60525874dc4efeb658433f1f253d54e0cba2d4"
        },
        "date": 1660159682993,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 485449124,
            "range": "± 13395787",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 525239952,
            "range": "± 13468952",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 963461511,
            "range": "± 74611207",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 330626748,
            "range": "± 5147996",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 4034815301,
            "range": "± 4994442569",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 10336402691,
            "range": "± 5578670",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 9076840,
            "range": "± 10540962",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 1388955074,
            "range": "± 421527181",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 5291275502,
            "range": "± 1253492679",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 142937806,
            "range": "± 54468772",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 1308657453,
            "range": "± 321524348",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 4439983663,
            "range": "± 600588600",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 10762193,
            "range": "± 301122",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 95587879,
            "range": "± 28430810",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 345958105,
            "range": "± 27250591",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 10659830,
            "range": "± 138649",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 96728836,
            "range": "± 18935949",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 375536366,
            "range": "± 26969166",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 20953463,
            "range": "± 722226",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joshuef@gmail.com",
            "name": "Josh Wilson",
            "username": "joshuef"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "f4b89d390eaeae0ab6dd329c1a0e9bbc65ec28a6",
          "message": "fix: update prefixmap getter call after name change",
          "timestamp": "2022-08-11T09:03:17+02:00",
          "tree_id": "ca7e0d938c23a1cda1c302ff394445fce5efb2b4",
          "url": "https://github.com/maidsafe/safe_network/commit/f4b89d390eaeae0ab6dd329c1a0e9bbc65ec28a6"
        },
        "date": 1660203904854,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 484772324,
            "range": "± 10545280",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 513550341,
            "range": "± 17437152",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 953171068,
            "range": "± 70660640",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 325100251,
            "range": "± 4529070",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 331851409,
            "range": "± 7745780",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 786956229,
            "range": "± 50272069",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 8768485,
            "range": "± 3460505",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 470613975,
            "range": "± 109244422",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 5826208724,
            "range": "± 910814424",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 140784420,
            "range": "± 47456855",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 1516635019,
            "range": "± 434234982",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 5144615483,
            "range": "± 678866337",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 10920013,
            "range": "± 236235",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 95097145,
            "range": "± 17344199",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 327525419,
            "range": "± 27478819",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 10703993,
            "range": "± 184453",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 94480655,
            "range": "± 2954722",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 342741601,
            "range": "± 27255453",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 21337127,
            "range": "± 338255",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joshuef@gmail.com",
            "name": "Josh Wilson",
            "username": "joshuef"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "aea5782e583ae353566abb0f10d94132bd9b14fe",
          "message": "chore: print full error during node startup fail",
          "timestamp": "2022-08-11T10:56:41+02:00",
          "tree_id": "ed4d37793c869b183d91e0b49805d89d892baaaf",
          "url": "https://github.com/maidsafe/safe_network/commit/aea5782e583ae353566abb0f10d94132bd9b14fe"
        },
        "date": 1660210710605,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 486447844,
            "range": "± 10233850",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 513812179,
            "range": "± 16612634",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 954278508,
            "range": "± 79934403",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 327043104,
            "range": "± 5893003",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 330731227,
            "range": "± 8387972",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 775629298,
            "range": "± 55730885",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 9770188,
            "range": "± 1608091",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 691094490,
            "range": "± 202241790",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 5147768951,
            "range": "± 1216979612",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 155702813,
            "range": "± 50641878",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 1330594338,
            "range": "± 508599318",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 5232225520,
            "range": "± 439402603",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 11177738,
            "range": "± 540872",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 95397653,
            "range": "± 46785108",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 342903801,
            "range": "± 24951218",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 10950350,
            "range": "± 146881",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 94458342,
            "range": "± 8761745",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 346329854,
            "range": "± 25514089",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 24136500,
            "range": "± 324778",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "yogeshwar_1997@hotmail.com",
            "name": "Yoga07",
            "username": "Yoga07"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "1af888c09b2f5a49d04a7068b7f948cf096da8f3",
          "message": "chore: add README docs for join process and traceroute",
          "timestamp": "2022-08-12T10:21:49+02:00",
          "tree_id": "c88401d676979d0d3b5fac6ac5d49fd1b8992a8c",
          "url": "https://github.com/maidsafe/safe_network/commit/1af888c09b2f5a49d04a7068b7f948cf096da8f3"
        },
        "date": 1660297934095,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 436411403,
            "range": "± 311444798",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 494972381,
            "range": "± 184649176",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 789821452,
            "range": "± 1506650171",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 333187166,
            "range": "± 19113788",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 10209404338,
            "range": "± 4036382276",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 10400482770,
            "range": "± 13996065",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 10631303,
            "range": "± 3594055",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 1468346381,
            "range": "± 379798298",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 5701413473,
            "range": "± 1219300193",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 114453480,
            "range": "± 37781425",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 1290340783,
            "range": "± 260400566",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 4548022062,
            "range": "± 526160588",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 14040076,
            "range": "± 303292",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 110422626,
            "range": "± 8557033",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 411514215,
            "range": "± 35138385",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 12172345,
            "range": "± 596471",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 104407463,
            "range": "± 28114581",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 410008266,
            "range": "± 27578856",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 25553581,
            "range": "± 1392212",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joshuef@gmail.com",
            "name": "Josh Wilson",
            "username": "joshuef"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "66c26782759be707edb922daa548e3f0a3f9be8c",
          "message": "chore: add partial eq for rust 1.63; dep updates",
          "timestamp": "2022-08-12T11:38:07+02:00",
          "tree_id": "b34d78b3cf1fdff476af4498f6c00c41d35465de",
          "url": "https://github.com/maidsafe/safe_network/commit/66c26782759be707edb922daa548e3f0a3f9be8c"
        },
        "date": 1660300505347,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 487553628,
            "range": "± 7363740",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 527403665,
            "range": "± 13917775",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 974918320,
            "range": "± 80320509",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 331031771,
            "range": "± 5825371",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 337194816,
            "range": "± 4975628",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 779618655,
            "range": "± 55007586",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 11574355,
            "range": "± 5807769",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 1355993650,
            "range": "± 425910327",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 5958079352,
            "range": "± 1578814618",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 100351548,
            "range": "± 52448669",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 1109526901,
            "range": "± 202784680",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 4747734081,
            "range": "± 422176733",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 12172394,
            "range": "± 366049",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 105751186,
            "range": "± 42543366",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 403294390,
            "range": "± 36226865",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 11683987,
            "range": "± 425310",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 105241729,
            "range": "± 10044672",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 408319139,
            "range": "± 29845189",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 28113198,
            "range": "± 831942",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joshuef@gmail.com",
            "name": "Josh Wilson",
            "username": "joshuef"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "a4a39b421103af7c143280ad3860b3cbd3016386",
          "message": "chore: further tweak dysf, reduce score by std dev for better avg.\n\nAlso adjusts tests to this, which now feels a bit saner too",
          "timestamp": "2022-08-12T12:29:09+02:00",
          "tree_id": "288ec5a4d2fdbdf4b93f1ebd4736ab9e59dc9509",
          "url": "https://github.com/maidsafe/safe_network/commit/a4a39b421103af7c143280ad3860b3cbd3016386"
        },
        "date": 1660303275328,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 475661072,
            "range": "± 5609762",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 500524092,
            "range": "± 13631982",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 935564464,
            "range": "± 78149619",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 322017370,
            "range": "± 6052727",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 333062320,
            "range": "± 11927533",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 781223015,
            "range": "± 53289488",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 7671235,
            "range": "± 4133435",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 2214013948,
            "range": "± 560582753",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 8624437041,
            "range": "± 1491185560",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 196293107,
            "range": "± 117758744",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 1793973198,
            "range": "± 515518377",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 7333541516,
            "range": "± 722837892",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 5966187,
            "range": "± 94018",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 53689308,
            "range": "± 27847832",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 281588662,
            "range": "± 56450940",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 5839411,
            "range": "± 55794",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 54611086,
            "range": "± 1469926",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 271456702,
            "range": "± 46089485",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 17101070,
            "range": "± 101889",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "action@github.com",
            "name": "GitHub Action",
            "username": "actions-user"
          },
          "committer": {
            "email": "action@github.com",
            "name": "GitHub Action",
            "username": "actions-user"
          },
          "distinct": true,
          "id": "53f60c2327f8a69f0b2ef6d1a4e96644c10aa358",
          "message": "chore(release): sn_interface-0.9.0/sn_dysfunction-0.8.0/sn_client-0.69.0/sn_node-0.65.0/sn_api-0.67.0/sn_cli-0.60.0",
          "timestamp": "2022-08-14T05:37:49Z",
          "tree_id": "f8ede8ab83b0466e6a9602552ce8334a00ceeda0",
          "url": "https://github.com/maidsafe/safe_network/commit/53f60c2327f8a69f0b2ef6d1a4e96644c10aa358"
        },
        "date": 1660458564309,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 477878174,
            "range": "± 9123992",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 511708416,
            "range": "± 14923608",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 944676314,
            "range": "± 58952815",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 325561897,
            "range": "± 4546514",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 335936465,
            "range": "± 3181381",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 784002110,
            "range": "± 18044357",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 8709804,
            "range": "± 1986056",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 1307464974,
            "range": "± 263730154",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 6058057854,
            "range": "± 1796188218",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 124036527,
            "range": "± 81450889",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 1127069509,
            "range": "± 200305267",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 4134522092,
            "range": "± 677671331",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 11540104,
            "range": "± 283752",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 98163450,
            "range": "± 12784760",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 364228796,
            "range": "± 27952946",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 12131547,
            "range": "± 324361",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 97616995,
            "range": "± 8951780",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 358653209,
            "range": "± 28471114",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 21731566,
            "range": "± 601787",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joshuef@gmail.com",
            "name": "Josh Wilson",
            "username": "joshuef"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "7c11b1ea35770a2211ee4afc746bbafedb02caf8",
          "message": "chore: dont have adults responding to AeProbe msgs that come through",
          "timestamp": "2022-08-15T08:33:21+02:00",
          "tree_id": "5da10af7164cc7e20ff74cdc0003d0dee0d7b2b3",
          "url": "https://github.com/maidsafe/safe_network/commit/7c11b1ea35770a2211ee4afc746bbafedb02caf8"
        },
        "date": 1660547990671,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 487312622,
            "range": "± 13964499",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 503627130,
            "range": "± 17312046",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 968873549,
            "range": "± 32096872",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 327277076,
            "range": "± 13232456",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 334229726,
            "range": "± 6892601",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 790553118,
            "range": "± 18199475",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 9251034,
            "range": "± 1420732",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 702376849,
            "range": "± 198017144",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 6995293284,
            "range": "± 1697061614",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 314649051,
            "range": "± 126276006",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 2457016416,
            "range": "± 707224482",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 7746299825,
            "range": "± 2323191522",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 12512811,
            "range": "± 300350",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 100528216,
            "range": "± 4658205",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 400895422,
            "range": "± 24948243",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 12481554,
            "range": "± 1840446",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 102571878,
            "range": "± 6079192",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 350826127,
            "range": "± 24840306",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 27921587,
            "range": "± 429049",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joshuef@gmail.com",
            "name": "Josh Wilson",
            "username": "joshuef"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "6471eb88f7ce8c060909930ac23c855f30e8690a",
          "message": "chore: retry more times for connection fails w/ client",
          "timestamp": "2022-08-15T08:58:35+02:00",
          "tree_id": "29b4c27f12f89d7a56ed0219066551196c1ff867",
          "url": "https://github.com/maidsafe/safe_network/commit/6471eb88f7ce8c060909930ac23c855f30e8690a"
        },
        "date": 1660549429415,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 474595717,
            "range": "± 10717085",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 509879750,
            "range": "± 15381633",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 990139831,
            "range": "± 57772569",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 328163850,
            "range": "± 7461630",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 342083817,
            "range": "± 4483354",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 801136469,
            "range": "± 67580325",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 9176947,
            "range": "± 1614398",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 1398393578,
            "range": "± 469664126",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 7417232215,
            "range": "± 1027200689",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 248851992,
            "range": "± 64271921",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 2436457608,
            "range": "± 558973100",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 4917621719,
            "range": "± 748472193",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 12252972,
            "range": "± 724795",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 100688660,
            "range": "± 10594762",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 395007852,
            "range": "± 42278216",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 12669212,
            "range": "± 4842609",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 108170529,
            "range": "± 15298541",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 376552927,
            "range": "± 40239806",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 25305171,
            "range": "± 862925",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "bzeeman@live.nl",
            "name": "Benno Zeeman",
            "username": "b-zee"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "d529bd61de83795b2b10cce12549374cd9521a4f",
          "message": "fix: add fallback if only single prefix",
          "timestamp": "2022-08-15T12:57:52+02:00",
          "tree_id": "99bb5cadd6506061bade964f0be83b5f89146cef",
          "url": "https://github.com/maidsafe/safe_network/commit/d529bd61de83795b2b10cce12549374cd9521a4f"
        },
        "date": 1660564536889,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 482149083,
            "range": "± 10704535",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 503808306,
            "range": "± 468912664",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 968861895,
            "range": "± 112287359",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 323761427,
            "range": "± 9311814",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 338766551,
            "range": "± 5389924",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 797549478,
            "range": "± 4175167411",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 11412354,
            "range": "± 3244140",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 1645590977,
            "range": "± 292967655",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 6821210690,
            "range": "± 1165648437",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 262940502,
            "range": "± 126394227",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 2683459673,
            "range": "± 567122019",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 6007197643,
            "range": "± 2335672019",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 14250942,
            "range": "± 932462",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 120669378,
            "range": "± 33102116",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 446449253,
            "range": "± 30465157",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 15236995,
            "range": "± 1063801",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 121450627,
            "range": "± 5812596",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 440798051,
            "range": "± 20534322",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 31036259,
            "range": "± 1351899",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joshuef@gmail.com",
            "name": "Josh Wilson",
            "username": "joshuef"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "b2c1cd4f32c54c249aaaf932df014f50268bed0c",
          "message": "chore(node): do not merge client requests to different adult indexes",
          "timestamp": "2022-08-15T19:10:24+02:00",
          "tree_id": "c0ff4fede002b3c542dfc4732e75b28f0a307098",
          "url": "https://github.com/maidsafe/safe_network/commit/b2c1cd4f32c54c249aaaf932df014f50268bed0c"
        },
        "date": 1660587815609,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 479431645,
            "range": "± 11302632",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 502083609,
            "range": "± 68650241",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 993845489,
            "range": "± 69534726",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 10181080992,
            "range": "± 4962745478",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 10200297318,
            "range": "± 29041337",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 10363866766,
            "range": "± 47484225",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 9282398,
            "range": "± 4892151",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 635992750,
            "range": "± 178368586",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 6887001063,
            "range": "± 1330334342",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 337104425,
            "range": "± 89417806",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 2397909517,
            "range": "± 400026498",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 4712840376,
            "range": "± 1931129165",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 11281120,
            "range": "± 689517",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 99984268,
            "range": "± 25986650",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 418025013,
            "range": "± 23648416",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 11077305,
            "range": "± 1377533",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 101936926,
            "range": "± 9739242",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 398967879,
            "range": "± 33976040",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 24013393,
            "range": "± 946468",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joshuef@gmail.com",
            "name": "Josh Wilson",
            "username": "joshuef"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "2936bf28e56e0086e687bd99979aa4b1c3bde1e3",
          "message": "chore: initialise flow control earlier",
          "timestamp": "2022-08-15T20:36:28+02:00",
          "tree_id": "727c406d6cfc63c1b7ca7ec3a7afb426f950a871",
          "url": "https://github.com/maidsafe/safe_network/commit/2936bf28e56e0086e687bd99979aa4b1c3bde1e3"
        },
        "date": 1660591203175,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 490119753,
            "range": "± 14999039",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 542953936,
            "range": "± 268188110",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 1198133865,
            "range": "± 43766892",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 323855945,
            "range": "± 15982302",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 335850228,
            "range": "± 8291431",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 810170344,
            "range": "± 58476616",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 8941161,
            "range": "± 3980352",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 640492910,
            "range": "± 165381993",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 6986904059,
            "range": "± 1325660378",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 234828655,
            "range": "± 75230235",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 2597676063,
            "range": "± 583311504",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 8447184109,
            "range": "± 2121043310",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 10864657,
            "range": "± 279533",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 97454732,
            "range": "± 10350817",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 366360387,
            "range": "± 29164319",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 11185898,
            "range": "± 268593",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 97111325,
            "range": "± 6626104",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 341524943,
            "range": "± 28833486",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 21883533,
            "range": "± 432603",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joshuef@gmail.com",
            "name": "Josh Wilson",
            "username": "joshuef"
          },
          "committer": {
            "email": "davidrusu.me@gmail.com",
            "name": "davidrusu",
            "username": "davidrusu"
          },
          "distinct": true,
          "id": "9f64d681e285de57a54f571e98ff68f1bf39b6f1",
          "message": "chore(node): increase data query limit\n\nNow we differentiate queries per adult/index, we may need more queries.",
          "timestamp": "2022-08-15T16:25:55-04:00",
          "tree_id": "e7f328ce03ef4ce23fafa5c2eaa95ed683562c34",
          "url": "https://github.com/maidsafe/safe_network/commit/9f64d681e285de57a54f571e98ff68f1bf39b6f1"
        },
        "date": 1660598509528,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 606412597,
            "range": "± 148447410",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 779938100,
            "range": "± 228187932",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 1238215728,
            "range": "± 1008497760",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 304932305,
            "range": "± 111982348",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 339899389,
            "range": "± 7257130",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 853566012,
            "range": "± 58317977",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 9888175,
            "range": "± 13383212",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 2289876342,
            "range": "± 371604396",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 11497681799,
            "range": "± 1830621830",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 388656225,
            "range": "± 194616800",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 3315463235,
            "range": "± 617669557",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 13749005244,
            "range": "± 1919035726",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 10156632,
            "range": "± 965968",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 90794681,
            "range": "± 28136028",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 421378191,
            "range": "± 95303073",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 10102915,
            "range": "± 729162",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 83214709,
            "range": "± 4944943",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 444037179,
            "range": "± 103259074",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 22822763,
            "range": "± 984643",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "293741+willief@users.noreply.github.com",
            "name": "Southside",
            "username": "willief"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "0af715e7f647ccae745c8adb41119be66af109a9",
          "message": "Update README.md\n\ntypo     \"ndoe\" for \"node\"",
          "timestamp": "2022-08-16T09:14:37+02:00",
          "tree_id": "7f84ca966d5f86404911569feb060ac8f5fba4b5",
          "url": "https://github.com/maidsafe/safe_network/commit/0af715e7f647ccae745c8adb41119be66af109a9"
        },
        "date": 1660637227382,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 499320180,
            "range": "± 10277453",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 548916136,
            "range": "± 156901139",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 1171804072,
            "range": "± 773211796",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 320883026,
            "range": "± 713864520",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 334205817,
            "range": "± 3570232020",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 10348025272,
            "range": "± 12485370",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 8891360,
            "range": "± 3929630",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 1452011698,
            "range": "± 479741820",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 7016636478,
            "range": "± 1551671707",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 318118726,
            "range": "± 82316041",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 1444107222,
            "range": "± 814786934",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 4994905541,
            "range": "± 406862994",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 11137172,
            "range": "± 120492",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 98260757,
            "range": "± 3998108",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 355192316,
            "range": "± 29056445",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 11058442,
            "range": "± 267187",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 101756000,
            "range": "± 15452563",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 364137338,
            "range": "± 32974983",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 22460377,
            "range": "± 222985",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joshuef@gmail.com",
            "name": "Josh Wilson",
            "username": "joshuef"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "06f5b607cdfbacba082612965630249e3c0f7300",
          "message": "tests(client): remove long lived client conn test\n\nNo longer relevant, client conns can be cleaned up by nodes every X time.\nSo clients have to be resilient and retry (which they do). So this (long)\ntest can be dropped",
          "timestamp": "2022-08-16T12:43:26+02:00",
          "tree_id": "fb6a30d4401955f97f46b18332e64a84a2f6aeb9",
          "url": "https://github.com/maidsafe/safe_network/commit/06f5b607cdfbacba082612965630249e3c0f7300"
        },
        "date": 1660649083907,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 492932096,
            "range": "± 11191687",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 531009076,
            "range": "± 11747078",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 1180122785,
            "range": "± 27259741",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 311455519,
            "range": "± 15669105",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 330121757,
            "range": "± 5211948",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 771317887,
            "range": "± 23149430",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 9205182,
            "range": "± 1085101",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 1336653611,
            "range": "± 397973417",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 6130308966,
            "range": "± 1230350573",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 206472036,
            "range": "± 55473042",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 1039181916,
            "range": "± 824010066",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 4985533908,
            "range": "± 880529711",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 11052935,
            "range": "± 236408",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 96608959,
            "range": "± 5675694",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 341137006,
            "range": "± 29958799",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 10995108,
            "range": "± 160529",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 93953836,
            "range": "± 8458965",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 329511151,
            "range": "± 27958517",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 24250016,
            "range": "± 266482",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "bzeeman@live.nl",
            "name": "Benno Zeeman",
            "username": "b-zee"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "2f8f8ca6ba0f2faae5bb4631c708988edf907725",
          "message": "chore(client): associated functions to methods",
          "timestamp": "2022-08-17T13:33:48+02:00",
          "tree_id": "fd7e87c9ed8f9e48c97f2addde76462e8546b92f",
          "url": "https://github.com/maidsafe/safe_network/commit/2f8f8ca6ba0f2faae5bb4631c708988edf907725"
        },
        "date": 1660739376722,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 501051442,
            "range": "± 12544248",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 550893872,
            "range": "± 11715519",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 1262823860,
            "range": "± 36335449",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 312843333,
            "range": "± 17433239",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 337504003,
            "range": "± 118426842",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 828060215,
            "range": "± 56598471",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 9708522,
            "range": "± 1597224",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 1488547464,
            "range": "± 259020368",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 5965435499,
            "range": "± 1138686461",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 199607557,
            "range": "± 111467509",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 2090226802,
            "range": "± 499900840",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 5371458375,
            "range": "± 816226050",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 12289387,
            "range": "± 347074",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 105532543,
            "range": "± 6853171",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 406354164,
            "range": "± 115603704",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 11891888,
            "range": "± 545297",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 102834198,
            "range": "± 4805089",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 401523812,
            "range": "± 31172979",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 24882265,
            "range": "± 733434",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joshuef@gmail.com",
            "name": "Josh Wilson",
            "username": "joshuef"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "7691f087b30805d68614581aa43b3d6933cd83c9",
          "message": "chore: refactor flow ctrl msg and cmd processing",
          "timestamp": "2022-08-22T09:36:14+02:00",
          "tree_id": "0b6d7a2b1e594d4f596b9ea071af017eb52a9d70",
          "url": "https://github.com/maidsafe/safe_network/commit/7691f087b30805d68614581aa43b3d6933cd83c9"
        },
        "date": 1661157388463,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 1946476048,
            "range": "± 3904285783",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 2010092841,
            "range": "± 486563234",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 11832677077,
            "range": "± 5459577414",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 477200680,
            "range": "± 4577131698",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 480268634,
            "range": "± 2750646",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 857034902,
            "range": "± 9496446",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 9037833,
            "range": "± 1288821",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 1346835088,
            "range": "± 412683497",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 5123843766,
            "range": "± 1441962087",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 205681364,
            "range": "± 50031790",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 2461255409,
            "range": "± 663831889",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 7731465400,
            "range": "± 1421121344",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 11272628,
            "range": "± 204490",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 95755718,
            "range": "± 49576256",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 331009295,
            "range": "± 165470125",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 11111508,
            "range": "± 245937",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 94190081,
            "range": "± 4238169",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 365223556,
            "range": "± 27818825",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 25177013,
            "range": "± 486672",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joshuef@gmail.com",
            "name": "Josh Wilson",
            "username": "joshuef"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "c8517a481e39bf688041cd8f8661bc663ee7bce7",
          "message": "chore(node): fix clippy some/none issues",
          "timestamp": "2022-08-22T09:40:50+02:00",
          "tree_id": "489616f1ed6ae82f8ff80f83b845cccf4d9a6473",
          "url": "https://github.com/maidsafe/safe_network/commit/c8517a481e39bf688041cd8f8661bc663ee7bce7"
        },
        "date": 1661157949318,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 1933306377,
            "range": "± 123923106",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 2051133926,
            "range": "± 3370960681",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 2208868464,
            "range": "± 642858268",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 468358852,
            "range": "± 461722574",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 490701036,
            "range": "± 486972296",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 842586005,
            "range": "± 1760272567",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 8950590,
            "range": "± 1551141",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 1306120577,
            "range": "± 351644745",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 6267052152,
            "range": "± 1174151156",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 231763007,
            "range": "± 132392554",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 2073033853,
            "range": "± 570126167",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 6704193502,
            "range": "± 2046572192",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 10913937,
            "range": "± 263404",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 107988218,
            "range": "± 21961756",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 469539121,
            "range": "± 96733167",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 11032793,
            "range": "± 428193",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 105658382,
            "range": "± 5542808",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 411198636,
            "range": "± 22543638",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 22727975,
            "range": "± 514476",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "joshuef@gmail.com",
            "name": "Josh Wilson",
            "username": "joshuef"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "0ae61c2877df283dde6f18800a40fc0e3afd603e",
          "message": "chore(node): continue with periodics afer process batch has been done",
          "timestamp": "2022-08-22T13:33:00+02:00",
          "tree_id": "dd040e1936dcf9957f43a5b55bb6aea96de89879",
          "url": "https://github.com/maidsafe/safe_network/commit/0ae61c2877df283dde6f18800a40fc0e3afd603e"
        },
        "date": 1661171549258,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 1926297731,
            "range": "± 156790697",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 1990923166,
            "range": "± 4065948537",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 2060576379,
            "range": "± 1241659426",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 474531716,
            "range": "± 2026530438",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 489340143,
            "range": "± 1609410256",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 828109610,
            "range": "± 81106215",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 8357718,
            "range": "± 1337072",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 1110687367,
            "range": "± 251494226",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 6295949082,
            "range": "± 1092425674",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 187826959,
            "range": "± 85749651",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 2316162370,
            "range": "± 658349482",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 8623236405,
            "range": "± 560538968",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 10991123,
            "range": "± 142545",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 99799326,
            "range": "± 11603763",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 335536660,
            "range": "± 56344903",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 10774266,
            "range": "± 251891",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 97702270,
            "range": "± 3994431",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 378240731,
            "range": "± 36431598",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 22518124,
            "range": "± 667094",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "RolandSherwin@protonmail.com",
            "name": "RolandSherwin",
            "username": "RolandSherwin"
          },
          "committer": {
            "email": "joshuef@gmail.com",
            "name": "joshuef",
            "username": "joshuef"
          },
          "distinct": true,
          "id": "2e370f5241bc26074526bc588f1f9bb34be574f2",
          "message": "fix(cli): copy default network_contacts when switching networks",
          "timestamp": "2022-08-22T14:26:41+02:00",
          "tree_id": "00e013c5ebe4e4561f752233809539fe3404ae56",
          "url": "https://github.com/maidsafe/safe_network/commit/2e370f5241bc26074526bc588f1f9bb34be574f2"
        },
        "date": 1661175175373,
        "tool": "cargo",
        "benches": [
          {
            "name": "upload-sampling/upload and read 3072b",
            "value": 1936642031,
            "range": "± 347927136",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 1mb",
            "value": 2786210340,
            "range": "± 3645721151",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload and read 10mb",
            "value": 2109517864,
            "range": "± 125880643",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 3072b",
            "value": 471566816,
            "range": "± 1251773621",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 1mb",
            "value": 492196335,
            "range": "± 1816097",
            "unit": "ns/iter"
          },
          {
            "name": "upload-sampling/upload 10mb",
            "value": 843275080,
            "range": "± 1640803105",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/100",
            "value": 11671452,
            "range": "± 1465197",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/1000",
            "value": 1681499586,
            "range": "± 324074918",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/register_writes/4000",
            "value": 6991555752,
            "range": "± 1180147154",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/100",
            "value": 240775252,
            "range": "± 88413121",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/1000",
            "value": 2737005289,
            "range": "± 637206664",
            "unit": "ns/iter"
          },
          {
            "name": "write-sampling/chunk writes/4000",
            "value": 9155838424,
            "range": "± 877391018",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/100",
            "value": 16727753,
            "range": "± 1134781",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/1000",
            "value": 135874853,
            "range": "± 31373690",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/register_keys/4000",
            "value": 496928977,
            "range": "± 146262541",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/100",
            "value": 17621416,
            "range": "± 1303360",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/1000",
            "value": 131706242,
            "range": "± 4947464",
            "unit": "ns/iter"
          },
          {
            "name": "read-sampling/chunk keys/4000",
            "value": 599824295,
            "range": "± 61833098",
            "unit": "ns/iter"
          },
          {
            "name": "generating keys",
            "value": 6288828,
            "range": "± 342366",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}