window.BENCHMARK_DATA = {
  "lastUpdate": 1650460484601,
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
      }
    ]
  }
}