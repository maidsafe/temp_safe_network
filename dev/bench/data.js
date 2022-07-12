window.BENCHMARK_DATA = {
  "lastUpdate": 1657606795767,
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
      }
    ]
  }
}