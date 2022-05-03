window.BENCHMARK_DATA = {
  "lastUpdate": 1651564568965,
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
      }
    ]
  }
}