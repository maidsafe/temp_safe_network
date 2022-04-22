window.BENCHMARK_DATA = {
  "lastUpdate": 1650647527276,
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
      }
    ]
  }
}