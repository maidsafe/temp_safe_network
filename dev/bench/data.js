window.BENCHMARK_DATA = {
  "lastUpdate": 1650265765952,
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
      }
    ]
  }
}