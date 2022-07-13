window.BENCHMARK_DATA = {
  "lastUpdate": 1657673341610,
  "repoUrl": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner",
  "entries": {
    "Rust Benchmark": [
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "c572355db8c19bc2cfc1f0a3a02d162e97816f44",
          "message": "Fix compile error\u0016",
          "timestamp": "2022-01-18T12:59:43+01:00",
          "tree_id": "addbc526d91d08e1ff008e02137a56ede33476b9",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/c572355db8c19bc2cfc1f0a3a02d162e97816f44"
        },
        "date": 1642507465658,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 181617,
            "range": "± 2597",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 2105044,
            "range": "± 28355",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 24080830,
            "range": "± 530726",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 23057149,
            "range": "± 350792",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 157777502,
            "range": "± 4905900",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "61bd63399041d3747187778e669b815f59e628e6",
          "message": "More efficient Pos/LexPos comparisons",
          "timestamp": "2022-01-18T13:08:31+01:00",
          "tree_id": "5c3d0a72a7ebb98f2facca237e4454dc2c5a1a99",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/61bd63399041d3747187778e669b815f59e628e6"
        },
        "date": 1642508017725,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 159209,
            "range": "± 1444",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 2033910,
            "range": "± 172851",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 26034366,
            "range": "± 111643",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 25308849,
            "range": "± 95536",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 244217692,
            "range": "± 4803278",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "1c32d31f29b97187a66bd9b00f72ab03ae1145d2",
          "message": "Move LogQueryContour and NaiveContours to their own files",
          "timestamp": "2022-01-18T13:09:36+01:00",
          "tree_id": "163a921ba6d11e908c7a9a668922f48fdfa83bd0",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/1c32d31f29b97187a66bd9b00f72ab03ae1145d2"
        },
        "date": 1642508072912,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 179186,
            "range": "± 6136",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1964523,
            "range": "± 22952",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 22619897,
            "range": "± 825959",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 22300826,
            "range": "± 754930",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 150650036,
            "range": "± 5867846",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "87db30b3b70e245382a11bfaa8a4e258772498fb",
          "message": "Add CentralContour, using caching of the last relevant dominant point",
          "timestamp": "2022-01-18T13:13:47+01:00",
          "tree_id": "10853eb81db935cbf3611a4ad13b709554e5c21f",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/87db30b3b70e245382a11bfaa8a4e258772498fb"
        },
        "date": 1642508467101,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 210171,
            "range": "± 6976",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 2278028,
            "range": "± 51183",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 26481742,
            "range": "± 1496123",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 26004968,
            "range": "± 1436831",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 166582766,
            "range": "± 9119355",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 115443,
            "range": "± 5065",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 1512224,
            "range": "± 39296",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 20215836,
            "range": "± 819477",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 21278341,
            "range": "± 1051461",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 168801332,
            "range": "± 9140881",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "e858886669507094908a885d3877e67d8b411d0e",
          "message": "Fix bug with USE_SHADOW_POINTS in NaiveContours\n\nWe must prune the point from multiple layers, not just the current one.",
          "timestamp": "2022-01-18T14:52:09+01:00",
          "tree_id": "b21a548407c3a9463f248aa1e715afb72bd5af5c",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/e858886669507094908a885d3877e67d8b411d0e"
        },
        "date": 1642514286240,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 176232,
            "range": "± 1920",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1302257,
            "range": "± 506770",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 18928007,
            "range": "± 341981",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 18014253,
            "range": "± 249004",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 149384938,
            "range": "± 7919950",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 96756,
            "range": "± 1017",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 1272317,
            "range": "± 14097",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 17096245,
            "range": "± 153319",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 17636009,
            "range": "± 150206",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 153628955,
            "range": "± 15238683",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "28b23774b69893137a544d6da4c70c73b6d183d9",
          "message": "Add HintContours and remove hints from NaiveContours",
          "timestamp": "2022-01-18T15:10:29+01:00",
          "tree_id": "9c20f56937a3d76a7285f521814a22662a345415",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/28b23774b69893137a544d6da4c70c73b6d183d9"
        },
        "date": 1642515429634,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 154441,
            "range": "± 2112",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1367262,
            "range": "± 18945",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 24141808,
            "range": "± 653102",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 23749614,
            "range": "± 495227",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 244281774,
            "range": "± 6237471",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 82839,
            "range": "± 819",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 1252945,
            "range": "± 12372",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 19462091,
            "range": "± 336609",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 19692011,
            "range": "± 305533",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 261031816,
            "range": "± 5982214",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "1d0c4c00da62eb8877d5231c424ff03969105e44",
          "message": "Increase threshold for failed benchmarks -- sadly it's not very accurate",
          "timestamp": "2022-01-18T15:19:02+01:00",
          "tree_id": "34cf8510918913975ab02a6cea2fbbdf55510a18",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/1d0c4c00da62eb8877d5231c424ff03969105e44"
        },
        "date": 1642515885880,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 175240,
            "range": "± 1986",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1282924,
            "range": "± 20045",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 21258615,
            "range": "± 2780930",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 20534278,
            "range": "± 272172",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 142892716,
            "range": "± 4320801",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 92902,
            "range": "± 748",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 1191112,
            "range": "± 8169",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 15567837,
            "range": "± 122390",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 15412024,
            "range": "± 135254",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 141868976,
            "range": "± 2499342",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "470fef49db9f09f4596cc7fe373e249ef846f278",
          "message": "Implement Hint & Stats for HintContours",
          "timestamp": "2022-01-18T19:17:56+01:00",
          "tree_id": "01c792abdce5ba4ebb167696daea9754f375022a",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/470fef49db9f09f4596cc7fe373e249ef846f278"
        },
        "date": 1642530223270,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 175424,
            "range": "± 2742",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1287385,
            "range": "± 11546",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 21237200,
            "range": "± 2708208",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 20574700,
            "range": "± 376298",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 143627101,
            "range": "± 4365549",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 92955,
            "range": "± 1073",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 1192044,
            "range": "± 8671",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 15586749,
            "range": "± 178292",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 15445992,
            "range": "± 117703",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 143719382,
            "range": "± 2788283",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "713a12e7f7d81ab380322bc562e3a7f60095506b",
          "message": "Update benchmark to use faster HintContours",
          "timestamp": "2022-01-18T19:18:48+01:00",
          "tree_id": "31d37afa0bd45fdd17046aea073421fd64db421c",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/713a12e7f7d81ab380322bc562e3a7f60095506b"
        },
        "date": 1642530269155,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 177324,
            "range": "± 2192",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1261747,
            "range": "± 11641",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 21425477,
            "range": "± 942124",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 20418213,
            "range": "± 829392",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 145737582,
            "range": "± 4449097",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 93279,
            "range": "± 1461",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 1105464,
            "range": "± 8836",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 13838361,
            "range": "± 193541",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 13141115,
            "range": "± 292231",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 137848398,
            "range": "± 4502256",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "8f5666ea604f40a58a773afa14fbce465a28f474",
          "message": "Add config setting to disable sorting of BucketQueue elements",
          "timestamp": "2022-01-18T19:41:20+01:00",
          "tree_id": "ed7c67a2d812a23c5ee1b2fe436d471cfa899c39",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/8f5666ea604f40a58a773afa14fbce465a28f474"
        },
        "date": 1642531705582,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 228809,
            "range": "± 30376",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1583698,
            "range": "± 1393339",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 21021962,
            "range": "± 2325325",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 20407469,
            "range": "± 2202980",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 160553353,
            "range": "± 8440284",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 119125,
            "range": "± 16210",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 1384703,
            "range": "± 188088",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 16223117,
            "range": "± 1356837",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 15448643,
            "range": "± 1785031",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 153110982,
            "range": "± 6697735",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "78d0134d5f5ddcdcb20eb9b8ad66863f26f56725",
          "message": "Add more stats to HintContours and some cleanup",
          "timestamp": "2022-01-18T21:14:07+01:00",
          "tree_id": "d73512cfd71d8dda57a0d5fc889aeb4b5e0dd86f",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/78d0134d5f5ddcdcb20eb9b8ad66863f26f56725"
        },
        "date": 1642537232553,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 155392,
            "range": "± 2440",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1127500,
            "range": "± 13785",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 19720936,
            "range": "± 1538807",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 18388068,
            "range": "± 735298",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 144422318,
            "range": "± 5732686",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 79376,
            "range": "± 995",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 973907,
            "range": "± 9049",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 12367601,
            "range": "± 586338",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 11767331,
            "range": "± 561358",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 141599718,
            "range": "± 6473778",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "49919a537e0062c6a151629157d3d3f217c8aafb",
          "message": "Replace HintContours::value_in_slice by binary search again",
          "timestamp": "2022-01-18T22:15:54+01:00",
          "tree_id": "54cbb2b264fb8814b4e76326492593e5f80fdd15",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/49919a537e0062c6a151629157d3d3f217c8aafb"
        },
        "date": 1642540901474,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 175248,
            "range": "± 1906",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1267634,
            "range": "± 4882",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 21624635,
            "range": "± 1128397",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 20601625,
            "range": "± 933421",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 151465417,
            "range": "± 6804980",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 92193,
            "range": "± 1476",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 1099571,
            "range": "± 11536",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 13547608,
            "range": "± 233718",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 12809901,
            "range": "± 511098",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 146027424,
            "range": "± 6354262",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "11cc158661e52ff20b4e2806a0bb477adbc0b198",
          "message": "Replace calls to h by h_with_hint in pruning",
          "timestamp": "2022-01-19T12:18:42+01:00",
          "tree_id": "5f37756a8b04aebdccdd29ac7810cee3ed4f1061",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/11cc158661e52ff20b4e2806a0bb477adbc0b198"
        },
        "date": 1642591414193,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 171718,
            "range": "± 1736",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1265789,
            "range": "± 18134",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 22234317,
            "range": "± 2466494",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 21057266,
            "range": "± 3224066",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 152614362,
            "range": "± 8236778",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "e49fa6b9046385d7498cc9dbd11cb47765615d87",
          "message": "Fix bugs in HintContours",
          "timestamp": "2022-01-19T12:19:13+01:00",
          "tree_id": "5f20d7ede40ef1fa88fea282e2788054e4e4a6a9",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/e49fa6b9046385d7498cc9dbd11cb47765615d87"
        },
        "date": 1642591544994,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 219418,
            "range": "± 45057",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1486158,
            "range": "± 469150",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 20120253,
            "range": "± 3241458",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 19571416,
            "range": "± 2346998",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 162663622,
            "range": "± 26486946",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 111662,
            "range": "± 23595",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 1213361,
            "range": "± 242879",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 14221885,
            "range": "± 2236253",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 13662568,
            "range": "± 3356401",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 143641123,
            "range": "± 11292087",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "e53b814baad5937933578eaad65a912d989268ff",
          "message": "Find seeds using Trie, both stack and dfs searching",
          "timestamp": "2022-01-19T12:43:09+01:00",
          "tree_id": "7cdd2a3992a22fd77aec0371a21d6ba20bc73db4",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/e53b814baad5937933578eaad65a912d989268ff"
        },
        "date": 1642592942218,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 178944,
            "range": "± 1794",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1293704,
            "range": "± 9488",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 21879137,
            "range": "± 483433",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 20656451,
            "range": "± 863789",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 154183659,
            "range": "± 6189672",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 94180,
            "range": "± 929",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 1118719,
            "range": "± 23604",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 13808667,
            "range": "± 125836",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 12989236,
            "range": "± 185857",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 147746163,
            "range": "± 6539672",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "0ba29339532113f5f22d657cd7b16d743c619037",
          "message": "Optimize greedy matching in A*",
          "timestamp": "2022-01-21T02:46:21+01:00",
          "tree_id": "77b5f4ffdafc06c4c0ef82f4ca828cdb1527200a",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/0ba29339532113f5f22d657cd7b16d743c619037"
        },
        "date": 1642729905124,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 236025,
            "range": "± 2079",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1179755,
            "range": "± 10480",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 20433955,
            "range": "± 2286263",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 19584117,
            "range": "± 2463252",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 141691068,
            "range": "± 4218761",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 91349,
            "range": "± 1553",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 1066663,
            "range": "± 7167",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 13515650,
            "range": "± 312224",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 12711504,
            "range": "± 195187",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 136215671,
            "range": "± 4894405",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "b574bc52ac3471d649c7f0842ac4578abc52a4fe",
          "message": "Remove DFS Trie search; a stack based approach is simpler",
          "timestamp": "2022-01-21T02:47:43+01:00",
          "tree_id": "d6c573d49217691f8fc2e900e9cdc22e65e2d659",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/b574bc52ac3471d649c7f0842ac4578abc52a4fe"
        },
        "date": 1642730021256,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 77033,
            "range": "± 9720",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1278367,
            "range": "± 158319",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 19696761,
            "range": "± 2450862",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 19943702,
            "range": "± 4481697",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 82699861,
            "range": "± 32147382",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 72098,
            "range": "± 16191",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 1180625,
            "range": "± 152086",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 17688311,
            "range": "± 3179276",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 16424506,
            "range": "± 1860119",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 78649572,
            "range": "± 10080526",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "144ef9659585014d72c98ef51132187e7f139db6",
          "message": "Remove f from State",
          "timestamp": "2022-01-21T03:22:03+01:00",
          "tree_id": "7b4678d8a52eaaea5828d5dff1f0ede272cad23f",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/144ef9659585014d72c98ef51132187e7f139db6"
        },
        "date": 1642732023358,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 80548,
            "range": "± 22965",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1406809,
            "range": "± 70162",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 20611101,
            "range": "± 287607",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 19840138,
            "range": "± 571871",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 89565849,
            "range": "± 9297214",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 77241,
            "range": "± 1067",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 1273735,
            "range": "± 10348",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 17271229,
            "range": "± 270098",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 16489408,
            "range": "± 256979",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 86469718,
            "range": "± 13175191",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "945473542ea038dbbf2f30aeac87a075d8e78d4f",
          "message": "Resize CostModel::MatchCost type",
          "timestamp": "2022-01-21T11:00:04+01:00",
          "tree_id": "ddd30a1bcde51dbaef0625a935b3d99b7821b43e",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/945473542ea038dbbf2f30aeac87a075d8e78d4f"
        },
        "date": 1642759759481,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 96201,
            "range": "± 2444",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1681120,
            "range": "± 19043",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 24915557,
            "range": "± 599739",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 23976658,
            "range": "± 1028787",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 105136253,
            "range": "± 4310097",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 90695,
            "range": "± 1505",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 1517016,
            "range": "± 11956",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 20752469,
            "range": "± 343681",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 19676274,
            "range": "± 419835",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 102128170,
            "range": "± 5514330",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "fe3867417d71850e661b1cec5e8d6c916449936a",
          "message": "Add find_matches benchmarks",
          "timestamp": "2022-01-21T11:17:49+01:00",
          "tree_id": "972b696bca9e34920b391ac8628ea7269e7e61bb",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/fe3867417d71850e661b1cec5e8d6c916449936a"
        },
        "date": 1642760689168,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 81632,
            "range": "± 22937",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1414419,
            "range": "± 260152",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 22171862,
            "range": "± 3760666",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 21255935,
            "range": "± 4135931",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 97048902,
            "range": "± 7731949",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 84153,
            "range": "± 13527",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 1333140,
            "range": "± 239544",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 18966815,
            "range": "± 3860096",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 19031615,
            "range": "± 2757869",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 90097740,
            "range": "± 18483783",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "d14c10d2cb7b1ef5b479dc0dfe1d6ad05b8833f8",
          "message": "Add installation instructions to snakemake",
          "timestamp": "2022-01-21T15:18:08+01:00",
          "tree_id": "9a69a6c6e869fdef3b9da0d20edff187bf390027",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/d14c10d2cb7b1ef5b479dc0dfe1d6ad05b8833f8"
        },
        "date": 1642775040087,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 81684,
            "range": "± 2547",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1403547,
            "range": "± 7227",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 20913553,
            "range": "± 511793",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 20245025,
            "range": "± 822113",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 99059625,
            "range": "± 5385899",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 77972,
            "range": "± 1678",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 1272385,
            "range": "± 7736",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 17598803,
            "range": "± 320956",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 16793154,
            "range": "± 290927",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 80439989,
            "range": "± 4901958",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "8b87e8fe2f218b3f56157ef7534901cb7d97b18c",
          "message": "Move double expand check into A* to save a hashmap insert/lookup",
          "timestamp": "2022-01-21T15:50:42+01:00",
          "tree_id": "330f33efc8e75338ea5886905d2a8deb65f213f8",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/8b87e8fe2f218b3f56157ef7534901cb7d97b18c"
        },
        "date": 1642776969702,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 67014,
            "range": "± 1067",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1208747,
            "range": "± 9136",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 17897963,
            "range": "± 175539",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 17125188,
            "range": "± 142756",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 78537877,
            "range": "± 4669690",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 62137,
            "range": "± 742",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 1096984,
            "range": "± 8825",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 15188667,
            "range": "± 95052",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 14410443,
            "range": "± 87554",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 73667332,
            "range": "± 3024711",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "dfddb5a610d1cd568053fa7898e2152b0bd6792e",
          "message": "Fix test: Add is_start_of_seed, prune_with_hint to EqualHeuristic",
          "timestamp": "2022-01-21T16:12:52+01:00",
          "tree_id": "1bbfe5cbee6ca948cdccefde2903e2b909290708",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/dfddb5a610d1cd568053fa7898e2152b0bd6792e"
        },
        "date": 1642778452159,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 81145,
            "range": "± 13631",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1611309,
            "range": "± 425335",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 24903118,
            "range": "± 3982243",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 23795035,
            "range": "± 4726035",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 110434019,
            "range": "± 16113209",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 83181,
            "range": "± 22242",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 1355339,
            "range": "± 198545",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 19924457,
            "range": "± 2559695",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 19453511,
            "range": "± 2165194",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 106459620,
            "range": "± 11491207",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "pesho@inf.ethz.ch",
            "name": "Pesho Ivanov",
            "username": "pesho-ivanov"
          },
          "committer": {
            "email": "pesho@inf.ethz.ch",
            "name": "Pesho Ivanov",
            "username": "pesho-ivanov"
          },
          "distinct": true,
          "id": "f992883a725ccf9372caea0bb65e7c8fb4b5c5db",
          "message": "refactor snakemake",
          "timestamp": "2022-01-21T17:46:59+01:00",
          "tree_id": "205e38def27ef2adb0a89ee3dbd54f7c69e9fa73",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/f992883a725ccf9372caea0bb65e7c8fb4b5c5db"
        },
        "date": 1642783963187,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 73665,
            "range": "± 889",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1409858,
            "range": "± 8900",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 20516285,
            "range": "± 521239",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 19723864,
            "range": "± 558203",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 95095337,
            "range": "± 7284123",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 71300,
            "range": "± 809",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 1257956,
            "range": "± 6602",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 17614945,
            "range": "± 381692",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 16897056,
            "range": "± 619974",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 91536064,
            "range": "± 10472023",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "2b2ed2364fafeda6008bd84e33881a2f8dc73f10",
          "message": "Pre-allocate more in DiagonalMap",
          "timestamp": "2022-01-21T18:10:03+01:00",
          "tree_id": "c46803a25ab0bcc24350fd4df8f841e32cb13aa7",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/2b2ed2364fafeda6008bd84e33881a2f8dc73f10"
        },
        "date": 1642785343929,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 74828,
            "range": "± 1068",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1354452,
            "range": "± 5139",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 20316941,
            "range": "± 293025",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 19423835,
            "range": "± 482115",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 88871755,
            "range": "± 4521001",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 67523,
            "range": "± 1051",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 1235376,
            "range": "± 5690",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 17132974,
            "range": "± 148102",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 16299390,
            "range": "± 201779",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 84398338,
            "range": "± 8243661",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "d7080b6a5e102a79f6dcd5e5f73672e82a79b2d3",
          "message": "Rename l -> k everywhere",
          "timestamp": "2022-01-21T19:31:43+01:00",
          "tree_id": "41e081d3a6e7514215eb39fc10a0fbc4785e83e7",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/d7080b6a5e102a79f6dcd5e5f73672e82a79b2d3"
        },
        "date": 1642790222721,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 65326,
            "range": "± 1098",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1196292,
            "range": "± 8042",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 17808838,
            "range": "± 73267",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 17066944,
            "range": "± 708206",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 73695089,
            "range": "± 4497507",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 60791,
            "range": "± 1815",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 1085293,
            "range": "± 46748",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 15011165,
            "range": "± 238076",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 14249730,
            "range": "± 192159",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 69964883,
            "range": "± 3406757",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "7f96dde0f7a04ed2468abe5637cf5627ff9e0d89",
          "message": "INEXACT MATCHES USING HASHING",
          "timestamp": "2022-01-22T15:36:21+01:00",
          "tree_id": "205de937a487d95c3cd7984dd90cb704e668486c",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/7f96dde0f7a04ed2468abe5637cf5627ff9e0d89"
        },
        "date": 1642862750134,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 40109,
            "range": "± 6255",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 891594,
            "range": "± 161246",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 14117260,
            "range": "± 1195298",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 13037864,
            "range": "± 1670132",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 45033217,
            "range": "± 5798440",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 41545,
            "range": "± 6860",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 824450,
            "range": "± 117156",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 10661646,
            "range": "± 1383252",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 9855657,
            "range": "± 1435415",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 42029163,
            "range": "± 5208692",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "067fb53a8fda725a3ca60066a320c248767203f2",
          "message": "DiagonalMap: indexing after grow can be unchecked",
          "timestamp": "2022-01-23T01:22:09+01:00",
          "tree_id": "c78ec7c9357ff93e18e44689c8c8e257144a325b",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/067fb53a8fda725a3ca60066a320c248767203f2"
        },
        "date": 1642897608875,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 34365,
            "range": "± 305",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 770204,
            "range": "± 18454",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 12170832,
            "range": "± 238586",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 11410406,
            "range": "± 190898",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 41829816,
            "range": "± 4180457",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 34922,
            "range": "± 348",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 669663,
            "range": "± 9994",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 9340637,
            "range": "± 43116",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 8601018,
            "range": "± 169645",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 39463888,
            "range": "± 7641731",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ivanov@pesho.info",
            "name": "Pesho Ivanov",
            "username": "pesho-ivanov"
          },
          "committer": {
            "email": "ivanov@pesho.info",
            "name": "Pesho Ivanov",
            "username": "pesho-ivanov"
          },
          "distinct": true,
          "id": "4a881a20bf208e1954c6d229f3816d151b5bd184",
          "message": "bench",
          "timestamp": "2022-01-23T20:12:56+01:00",
          "tree_id": "11aee598787b39c54fc6e359a06bbb1dff43c816",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/4a881a20bf208e1954c6d229f3816d151b5bd184"
        },
        "date": 1642965428849,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 37965,
            "range": "± 249",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 877275,
            "range": "± 7486",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 13905158,
            "range": "± 59761",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 12928141,
            "range": "± 90466",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 44871009,
            "range": "± 1844551",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 39950,
            "range": "± 293",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 768073,
            "range": "± 2864",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 10709560,
            "range": "± 22060",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 9823336,
            "range": "± 15364",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 41023593,
            "range": "± 2195065",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "5b489d41e8cb285992e336adbe87586529d411bc",
          "message": "Add table comparing all parameters",
          "timestamp": "2022-01-25T17:40:34+01:00",
          "tree_id": "015027e5ba3539e41f2a9040f37de64c674409aa",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/5b489d41e8cb285992e336adbe87586529d411bc"
        },
        "date": 1643129163065,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 38044,
            "range": "± 444",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 883581,
            "range": "± 5159",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 14596602,
            "range": "± 451090",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 13624859,
            "range": "± 528330",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 52345140,
            "range": "± 3715091",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 40596,
            "range": "± 200",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 772669,
            "range": "± 2597",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 11512613,
            "range": "± 764171",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 10420969,
            "range": "± 901343",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 50992754,
            "range": "± 3129465",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "c02a9c6223fc1c382427c8685dce85aac93bf7bd",
          "message": "Add table comparing all parameters",
          "timestamp": "2022-01-25T17:51:07+01:00",
          "tree_id": "22205eb97da8c32fd37b6576098b7c2b4b64e2c9",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/c02a9c6223fc1c382427c8685dce85aac93bf7bd"
        },
        "date": 1643129767484,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 45218,
            "range": "± 513",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1060704,
            "range": "± 7617",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 16892511,
            "range": "± 157749",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 15823536,
            "range": "± 278486",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 57774907,
            "range": "± 2100287",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 48011,
            "range": "± 279",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 924682,
            "range": "± 3828",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 13094737,
            "range": "± 436648",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 12089353,
            "range": "± 312017",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 54576409,
            "range": "± 3761599",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "9306d6c49adedb7d141b070f2df04973259b889a",
          "message": "Fix table for tool comparison and add it",
          "timestamp": "2022-01-25T18:04:13+01:00",
          "tree_id": "d3dd73f5b779cc54b6e8cfa4bf5552767c067917",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/9306d6c49adedb7d141b070f2df04973259b889a"
        },
        "date": 1643130633458,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 45549,
            "range": "± 281",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1062512,
            "range": "± 7501",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 17235821,
            "range": "± 541870",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 16134860,
            "range": "± 356269",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 59861575,
            "range": "± 1839400",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 48022,
            "range": "± 187",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 924562,
            "range": "± 3953",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 13580407,
            "range": "± 514355",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 12579659,
            "range": "± 432319",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 56774110,
            "range": "± 2319636",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ivanov@pesho.info",
            "name": "Pesho Ivanov",
            "username": "pesho-ivanov"
          },
          "committer": {
            "email": "ivanov@pesho.info",
            "name": "Pesho Ivanov",
            "username": "pesho-ivanov"
          },
          "distinct": true,
          "id": "98279b0f4b7319728f27afcda33d5a8d1c8b94eb",
          "message": "polyfit fixed",
          "timestamp": "2022-01-25T19:44:06+01:00",
          "tree_id": "b5ace4f77ffc6ac2742fdb40127cadcc6aa06681",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/98279b0f4b7319728f27afcda33d5a8d1c8b94eb"
        },
        "date": 1643136544662,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 45792,
            "range": "± 415",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1061590,
            "range": "± 9146",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 17323279,
            "range": "± 379781",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 16160580,
            "range": "± 527261",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 59873160,
            "range": "± 2244264",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 48122,
            "range": "± 219",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 925730,
            "range": "± 3940",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 13422174,
            "range": "± 661401",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 12265376,
            "range": "± 445132",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 56083661,
            "range": "± 3804531",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "9db2ff6abe8662f662ff3fdb7edd050948d12c95",
          "message": "Add Repeats to Snakemake benchmarks",
          "timestamp": "2022-01-26T11:59:28+01:00",
          "tree_id": "5404807898e01b9f051978b85ef0fd1b6b9f88fb",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/9db2ff6abe8662f662ff3fdb7edd050948d12c95"
        },
        "date": 1643195099862,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 38267,
            "range": "± 244",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 936863,
            "range": "± 8799",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 17719878,
            "range": "± 128450",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 17212354,
            "range": "± 73575",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 163520712,
            "range": "± 2879440",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 40150,
            "range": "± 405",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 777552,
            "range": "± 8500",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 14481339,
            "range": "± 172241",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 14085694,
            "range": "± 43168",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 178469600,
            "range": "± 1773977",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "6863f625aac55a010c2161efa90563cbbdab4214",
          "message": "New tables; update notebook",
          "timestamp": "2022-01-29T02:39:43+01:00",
          "tree_id": "d3c285b912b9753f2ddca9f1e8a42bee21c2af13",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/6863f625aac55a010c2161efa90563cbbdab4214"
        },
        "date": 1643420675405,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 36391,
            "range": "± 7304",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 782891,
            "range": "± 89493",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 12620394,
            "range": "± 1672316",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 12420034,
            "range": "± 2231998",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 45725454,
            "range": "± 4863069",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 39726,
            "range": "± 3952",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 700424,
            "range": "± 149811",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 10359522,
            "range": "± 2149128",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 9408995,
            "range": "± 1837242",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 45071009,
            "range": "± 7234586",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "2c2bff8922c9d58ae8e58e4dea3c1cd73e89ca6a",
          "message": "Use SPLIT VEC",
          "timestamp": "2022-01-30T13:46:26+01:00",
          "tree_id": "6c7f5ff1b3318cc5115c96b3811504e29e345394",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/2c2bff8922c9d58ae8e58e4dea3c1cd73e89ca6a"
        },
        "date": 1643559739235,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 45989,
            "range": "± 3374",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1058419,
            "range": "± 39408",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 16653553,
            "range": "± 515402",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 15507628,
            "range": "± 451549",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 56093836,
            "range": "± 2757356",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 50709,
            "range": "± 1092",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 936159,
            "range": "± 21938",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 13042824,
            "range": "± 396058",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 11793408,
            "range": "± 294140",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 42323069,
            "range": "± 3618792",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ivanov@pesho.info",
            "name": "Pesho Ivanov",
            "username": "pesho-ivanov"
          },
          "committer": {
            "email": "ivanov@pesho.info",
            "name": "Pesho Ivanov",
            "username": "pesho-ivanov"
          },
          "distinct": true,
          "id": "8367ce54fc12db1cd8763a32ddcac1f26a4e5a59",
          "message": "not yet there",
          "timestamp": "2022-01-31T12:01:47+01:00",
          "tree_id": "1e770ef5d5884e2f715cd1a5f9537c99ce570e45",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/8367ce54fc12db1cd8763a32ddcac1f26a4e5a59"
        },
        "date": 1643627205751,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 38558,
            "range": "± 14463",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 782580,
            "range": "± 6769",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 14954453,
            "range": "± 161972",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 14095762,
            "range": "± 283318",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 62043375,
            "range": "± 682106",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 41230,
            "range": "± 533",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 782560,
            "range": "± 3143",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 13654770,
            "range": "± 353095",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 12496789,
            "range": "± 278648",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 54201911,
            "range": "± 981364",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "01b66cd9a6f80a1f83fc7e0c54e62136ff0308f9",
          "message": "Fix bugs in snakemake; scale up to 100M",
          "timestamp": "2022-01-31T16:38:19+01:00",
          "tree_id": "9c5c489676c80d64d8dbafd09f1f9eccf94fa288",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/01b66cd9a6f80a1f83fc7e0c54e62136ff0308f9"
        },
        "date": 1643643800016,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 43238,
            "range": "± 3339",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1000978,
            "range": "± 91867",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 15961868,
            "range": "± 1380243",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 14518412,
            "range": "± 1369332",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 51131031,
            "range": "± 5273436",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 46398,
            "range": "± 4504",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 892403,
            "range": "± 94160",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 12502494,
            "range": "± 844551",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 11320164,
            "range": "± 826630",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 40007570,
            "range": "± 3043596",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "abb5937a43708b6abdefbd6f656a0dc4e3ff0ae0",
          "message": "Update e0.05-k14 table with astar stats",
          "timestamp": "2022-01-31T18:09:09+01:00",
          "tree_id": "9b76e4312f083ed237c6f9f35268273e9d9a142b",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/abb5937a43708b6abdefbd6f656a0dc4e3ff0ae0"
        },
        "date": 1643649210192,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 33511,
            "range": "± 315",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 784040,
            "range": "± 3947",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 12556452,
            "range": "± 330668",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 11696758,
            "range": "± 639163",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 46426008,
            "range": "± 3207604",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 36005,
            "range": "± 228",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 689709,
            "range": "± 2604",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 9661581,
            "range": "± 178570",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 8730568,
            "range": "± 132814",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 36259136,
            "range": "± 4482501",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "c455957672074f224dca05724f3a78973fe4efe4",
          "message": "Put astar stats in .stats file and .tsv files",
          "timestamp": "2022-01-31T18:15:10+01:00",
          "tree_id": "641b093d0646a6d63d5cea3a37ef404763b4db6d",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/c455957672074f224dca05724f3a78973fe4efe4"
        },
        "date": 1643649717455,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 45230,
            "range": "± 8639",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1017225,
            "range": "± 81200",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 16861228,
            "range": "± 1384221",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 15448185,
            "range": "± 1020637",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 63743122,
            "range": "± 5752382",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 50284,
            "range": "± 12357",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 921571,
            "range": "± 156935",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 13007036,
            "range": "± 820793",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 11897489,
            "range": "± 859207",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 48158902,
            "range": "± 6676594",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "88161e3d4d3285c658d8de1b226f3f11594f3222",
          "message": "Use own generate_dataset in snakemake instead of WFA binary",
          "timestamp": "2022-02-01T22:02:11+01:00",
          "tree_id": "68ff188a30acf1bc83961b4d83c1bf55c3665e72",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/88161e3d4d3285c658d8de1b226f3f11594f3222"
        },
        "date": 1643749645902,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 38052,
            "range": "± 205",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 931835,
            "range": "± 13165",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 17673062,
            "range": "± 331420",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 17228781,
            "range": "± 404547",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 165852605,
            "range": "± 1337270",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 40901,
            "range": "± 331",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 808486,
            "range": "± 7878",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 11477556,
            "range": "± 307316",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 10345198,
            "range": "± 260568",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 37392680,
            "range": "± 2565468",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "76aca167ef77d6de08abf84ea74f7ca09f81b274",
          "message": "Add table showing log linear growth",
          "timestamp": "2022-02-02T12:31:35+01:00",
          "tree_id": "125b07b316905b92dcf1c4e7b51324f4dd669ede",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/76aca167ef77d6de08abf84ea74f7ca09f81b274"
        },
        "date": 1643801795995,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 37848,
            "range": "± 298",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 921152,
            "range": "± 8435",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 17451659,
            "range": "± 428645",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 17172623,
            "range": "± 495901",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 165398328,
            "range": "± 1084575",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 40268,
            "range": "± 267",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 790075,
            "range": "± 12319",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 11295188,
            "range": "± 57798",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 10221244,
            "range": "± 44951",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 35115366,
            "range": "± 1412905",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "6c0bc9fbe3968a4417e07a95db1d714373548a2e",
          "message": "Latest version of notebook",
          "timestamp": "2022-02-02T16:08:03+01:00",
          "tree_id": "379831938716fda38522f70749e313a72cc05814",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/6c0bc9fbe3968a4417e07a95db1d714373548a2e"
        },
        "date": 1643814757267,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 38223,
            "range": "± 250",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 887811,
            "range": "± 5607",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 13967924,
            "range": "± 206793",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 13045051,
            "range": "± 77203",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 48615952,
            "range": "± 2848807",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 41103,
            "range": "± 287",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 779400,
            "range": "± 2698",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 10835772,
            "range": "± 150162",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 9794859,
            "range": "± 93105",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 38571978,
            "range": "± 4293525",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "b61cb19844737c67566c519f0043fa0c6182d6c9",
          "message": "Move data/* to evals/",
          "timestamp": "2022-02-02T16:15:18+01:00",
          "tree_id": "92fac38b0afe8d16105d02d7309d47037a56b953",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/b61cb19844737c67566c519f0043fa0c6182d6c9"
        },
        "date": 1643815240466,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 45853,
            "range": "± 1544",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1050519,
            "range": "± 60465",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 16797031,
            "range": "± 955322",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 15762993,
            "range": "± 584576",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 62536349,
            "range": "± 3194091",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 49573,
            "range": "± 1753",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 939031,
            "range": "± 7441",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 13090561,
            "range": "± 520512",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 11697656,
            "range": "± 729126",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 46063039,
            "range": "± 5725420",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "ae597d7f73db4ac0391ca557e83f53c62e47cfd8",
          "message": "Move snakefile into evals/",
          "timestamp": "2022-02-02T16:22:55+01:00",
          "tree_id": "67286bfe0ae5990e2638183139d24e9f4f7f885a",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/ae597d7f73db4ac0391ca557e83f53c62e47cfd8"
        },
        "date": 1643815619486,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 33743,
            "range": "± 198",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 782760,
            "range": "± 3114",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 13942676,
            "range": "± 224054",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 13020914,
            "range": "± 55902",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 44459405,
            "range": "± 934588",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 41096,
            "range": "± 225",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 781820,
            "range": "± 2834",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 10809344,
            "range": "± 21806",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 9781961,
            "range": "± 34828",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 34005683,
            "range": "± 751247",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ivanov@pesho.info",
            "name": "Pesho Ivanov",
            "username": "pesho-ivanov"
          },
          "committer": {
            "email": "ivanov@pesho.info",
            "name": "Pesho Ivanov",
            "username": "pesho-ivanov"
          },
          "distinct": true,
          "id": "8cfdc3c4c8c3e0b39347845ffcdeea9d2aaab823",
          "message": "notebooks cleanup",
          "timestamp": "2022-02-02T16:42:49+01:00",
          "tree_id": "86ffd127e891b51883afd115e65536c0a0878372",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/8cfdc3c4c8c3e0b39347845ffcdeea9d2aaab823"
        },
        "date": 1643816839035,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 33784,
            "range": "± 267",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 786079,
            "range": "± 5252",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 13530743,
            "range": "± 614665",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 12618326,
            "range": "± 640998",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 51791393,
            "range": "± 3705946",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 36499,
            "range": "± 215",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 690778,
            "range": "± 4046",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 11238787,
            "range": "± 1647144",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 9109492,
            "range": "± 595568",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 43446745,
            "range": "± 5734326",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "4ef107a4c6e1188a92d5d7cdb8258a3c9078ba96",
          "message": "Add a new N=1M params table",
          "timestamp": "2022-02-02T17:03:02+01:00",
          "tree_id": "41d30ae8bbda1e18f42ebf1db492dde51b60eae5",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/4ef107a4c6e1188a92d5d7cdb8258a3c9078ba96"
        },
        "date": 1643818088975,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 38069,
            "range": "± 189",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 921663,
            "range": "± 11463",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 17550113,
            "range": "± 414895",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 17180723,
            "range": "± 347181",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 165398832,
            "range": "± 1276041",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 40554,
            "range": "± 293",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 792158,
            "range": "± 9125",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 11391516,
            "range": "± 206367",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 10310068,
            "range": "± 232823",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 36082654,
            "range": "± 1830551",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ivanov@pesho.info",
            "name": "Pesho Ivanov",
            "username": "pesho-ivanov"
          },
          "committer": {
            "email": "ivanov@pesho.info",
            "name": "Pesho Ivanov",
            "username": "pesho-ivanov"
          },
          "distinct": true,
          "id": "3ce329309d75bbf12725477a494ce33614e70106",
          "message": "-evals/stats/",
          "timestamp": "2022-02-02T17:33:15+01:00",
          "tree_id": "b5321576ac7a4e7293b22ca9af39172ad6b50046",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/3ce329309d75bbf12725477a494ce33614e70106"
        },
        "date": 1643819858403,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 38321,
            "range": "± 494",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 891267,
            "range": "± 5249",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 14427533,
            "range": "± 392086",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 13387922,
            "range": "± 494772",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 51098212,
            "range": "± 3136971",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 41258,
            "range": "± 208",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 782884,
            "range": "± 2981",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 10959227,
            "range": "± 180200",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 9926548,
            "range": "± 310465",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 41191821,
            "range": "± 3567697",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "5a1d0d3a4f6563941932588ab72b44340f9ee63b",
          "message": "Better filtering for m=1, change k=32 to k=31; push new table",
          "timestamp": "2022-02-02T19:05:54+01:00",
          "tree_id": "39d8b9483e672992d174eb36fa03c61779bffc49",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/5a1d0d3a4f6563941932588ab72b44340f9ee63b"
        },
        "date": 1643825461598,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 37964,
            "range": "± 193",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 924181,
            "range": "± 8122",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 17975545,
            "range": "± 512513",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 17476785,
            "range": "± 413815",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 166961228,
            "range": "± 1270007",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 40027,
            "range": "± 245",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 786100,
            "range": "± 9923",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 11509546,
            "range": "± 609010",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 10336027,
            "range": "± 246850",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 37593051,
            "range": "± 2592421",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ivanov@pesho.info",
            "name": "Pesho Ivanov",
            "username": "pesho-ivanov"
          },
          "committer": {
            "email": "ivanov@pesho.info",
            "name": "Pesho Ivanov",
            "username": "pesho-ivanov"
          },
          "distinct": true,
          "id": "408f51b28224c4ae127dc095759b5febda58069b",
          "message": "adding back the notebooks",
          "timestamp": "2022-02-02T22:26:54+01:00",
          "tree_id": "93035407ef2bc167e587563c47d2e39f3563c741",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/408f51b28224c4ae127dc095759b5febda58069b"
        },
        "date": 1643837464522,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 38173,
            "range": "± 471",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 880385,
            "range": "± 2592",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 14260414,
            "range": "± 670072",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 13249085,
            "range": "± 584206",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 51468680,
            "range": "± 5948537",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 40434,
            "range": "± 394",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 772838,
            "range": "± 4337",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 10931751,
            "range": "± 315384",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 9891036,
            "range": "± 345476",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 37358447,
            "range": "± 6439555",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "pesho-ivanov@users.noreply.github.com",
            "name": "Pesho Ivanov",
            "username": "pesho-ivanov"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "4653004cdd77e3aee32ae496ece5f2c6f12ad666",
          "message": "capitalization :\"p",
          "timestamp": "2022-02-02T22:27:46+01:00",
          "tree_id": "99d73c822f8bec11eddfb94935d38cf39217b190",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/4653004cdd77e3aee32ae496ece5f2c6f12ad666"
        },
        "date": 1643837591744,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 43394,
            "range": "± 7472",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 939231,
            "range": "± 174992",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 15410792,
            "range": "± 2346462",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 14850566,
            "range": "± 2443112",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 63197340,
            "range": "± 9078097",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 51542,
            "range": "± 8132",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 965213,
            "range": "± 109928",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 13723454,
            "range": "± 2836603",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 12053394,
            "range": "± 1711708",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 43955508,
            "range": "± 7486246",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "0c2d256bb8e7b527929c6fdae74758729be67539",
          "message": "Add new plots to evals notebook",
          "timestamp": "2022-02-02T23:34:18+01:00",
          "tree_id": "f0b51faee9f77cd7bd1958bd53881f5e56cfd1dc",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/0c2d256bb8e7b527929c6fdae74758729be67539"
        },
        "date": 1643841538141,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 38159,
            "range": "± 360",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 884551,
            "range": "± 4943",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 14252117,
            "range": "± 380780",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 13260835,
            "range": "± 586854",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 51336949,
            "range": "± 2708854",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 40583,
            "range": "± 205",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 775269,
            "range": "± 3088",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 10934836,
            "range": "± 334805",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 9882263,
            "range": "± 348765",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 41962153,
            "range": "± 5356955",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "9f9788c775a58f4c5ce31172229207e4106747fc",
          "message": "Filter python notebook output. Add to .git/config:\n\n```\n[filter \"strip-notebook-output\"]\n    clean = \"jupyter nbconvert --ClearOutputPreprocessor.enabled=True --to=notebook --stdin --stdout --log-level=ERROR\"\n```",
          "timestamp": "2022-02-02T23:37:09+01:00",
          "tree_id": "085ac8ba3eccbb30b8aef6b2beb3545c3617d2fb",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/9f9788c775a58f4c5ce31172229207e4106747fc"
        },
        "date": 1643841773899,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 38737,
            "range": "± 541",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 882530,
            "range": "± 4886",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 14054829,
            "range": "± 50415",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 13049887,
            "range": "± 356501",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 46271246,
            "range": "± 3186336",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 40623,
            "range": "± 217",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 775389,
            "range": "± 2593",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 10841484,
            "range": "± 40437",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 9801371,
            "range": "± 33504",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 34990845,
            "range": "± 2061028",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "fa1324668780d534bc635a81e7920d004cca2964",
          "message": "Remove output from notebook",
          "timestamp": "2022-02-02T23:39:32+01:00",
          "tree_id": "b2c34662bdb5b5457e5569f8c40a014d54177b55",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/fa1324668780d534bc635a81e7920d004cca2964"
        },
        "date": 1643841830052,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 38684,
            "range": "± 577",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 882551,
            "range": "± 4677",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 14004868,
            "range": "± 125277",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 12991771,
            "range": "± 67169",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 44655861,
            "range": "± 2448295",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 40653,
            "range": "± 194",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 775640,
            "range": "± 2985",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 10786308,
            "range": "± 36641",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 9753515,
            "range": "± 28620",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 34075336,
            "range": "± 2377610",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ivanov@pesho.info",
            "name": "Pesho Ivanov",
            "username": "pesho-ivanov"
          },
          "committer": {
            "email": "ivanov@pesho.info",
            "name": "Pesho Ivanov",
            "username": "pesho-ivanov"
          },
          "distinct": true,
          "id": "f91974e3355bb8b95fa0b8393ce437770873b8f9",
          "message": "__pycache__",
          "timestamp": "2022-02-02T23:45:42+01:00",
          "tree_id": "df3d5b25167ddf67bf2ac0a815727c282a4e4eec",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/f91974e3355bb8b95fa0b8393ce437770873b8f9"
        },
        "date": 1643842207189,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 37832,
            "range": "± 196",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 880931,
            "range": "± 4942",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 14334481,
            "range": "± 589397",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 13248406,
            "range": "± 588040",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 49756514,
            "range": "± 1866095",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 40599,
            "range": "± 165",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 775200,
            "range": "± 3366",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 11027597,
            "range": "± 584817",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 10037844,
            "range": "± 262343",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 41184215,
            "range": "± 4151316",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ivanov@pesho.info",
            "name": "Pesho Ivanov",
            "username": "pesho-ivanov"
          },
          "committer": {
            "email": "ivanov@pesho.info",
            "name": "Pesho Ivanov",
            "username": "pesho-ivanov"
          },
          "distinct": true,
          "id": "790146c1d67e536d0442d5c03b72abc1d623b6a4",
          "message": "+heatmap (not yet working)",
          "timestamp": "2022-02-03T00:20:51+01:00",
          "tree_id": "d1ef3e588fb9a49eca699484f6364410f8396bfa",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/790146c1d67e536d0442d5c03b72abc1d623b6a4"
        },
        "date": 1643844345534,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 45758,
            "range": "± 3849",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1056106,
            "range": "± 62436",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 16788247,
            "range": "± 1107936",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 15660290,
            "range": "± 934887",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 57542548,
            "range": "± 4225647",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 48904,
            "range": "± 2188",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 928346,
            "range": "± 30699",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 12792256,
            "range": "± 1043865",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 11489048,
            "range": "± 840100",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 44191494,
            "range": "± 5873766",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "72d9956202d54f0bce6f87cdd8e09ade30e6c8c9",
          "message": "Stricter filtering for multiple matches per seed",
          "timestamp": "2022-02-03T00:26:56+01:00",
          "tree_id": "858e624692e4a065c090d96a9b014c071e09ca71",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/72d9956202d54f0bce6f87cdd8e09ade30e6c8c9"
        },
        "date": 1643844717926,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 37989,
            "range": "± 197",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 947967,
            "range": "± 12967",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 17757981,
            "range": "± 330992",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 17205860,
            "range": "± 259194",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 165744133,
            "range": "± 3003226",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 40624,
            "range": "± 282",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 790751,
            "range": "± 9019",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 11326568,
            "range": "± 256096",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 10309576,
            "range": "± 253669",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 35416665,
            "range": "± 1899771",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "6756df87a48a00386c4f900771b16012835cef4b",
          "message": "REDUCE RETRIES: propagate deleted contours to offset for priority queue",
          "timestamp": "2022-02-06T14:35:02+01:00",
          "tree_id": "9eec50972605a494a09c7417bd5f02273879045d",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/6756df87a48a00386c4f900771b16012835cef4b"
        },
        "date": 1644154768352,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 39055,
            "range": "± 323",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 897048,
            "range": "± 5694",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 15024991,
            "range": "± 729454",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 13834147,
            "range": "± 640344",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 55488511,
            "range": "± 3222789",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 41331,
            "range": "± 247",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 787496,
            "range": "± 3612",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 11311054,
            "range": "± 350510",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 10217734,
            "range": "± 329759",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 42463711,
            "range": "± 2431384",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ivanov@pesho.info",
            "name": "Pesho Ivanov",
            "username": "pesho-ivanov"
          },
          "committer": {
            "email": "ivanov@pesho.info",
            "name": "Pesho Ivanov",
            "username": "pesho-ivanov"
          },
          "distinct": true,
          "id": "999a8f6fe3b7f3a0ce4697cbd3f76e782a615baa",
          "message": "plots update",
          "timestamp": "2022-02-07T13:13:19+01:00",
          "tree_id": "107db9be0ce88043b58ee28ef1527609b02167b6",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/999a8f6fe3b7f3a0ce4697cbd3f76e782a615baa"
        },
        "date": 1644236269888,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 39698,
            "range": "± 283",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 895912,
            "range": "± 3592",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 15027977,
            "range": "± 746546",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 14084463,
            "range": "± 726580",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 54733121,
            "range": "± 2753791",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 42336,
            "range": "± 269",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 790350,
            "range": "± 2559",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 11193447,
            "range": "± 437138",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 10293425,
            "range": "± 608410",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 45965602,
            "range": "± 3477374",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "fdcbbbb2369346674ca9ca6945fac521f8abc089",
          "message": "Add num_filtered_matches stat",
          "timestamp": "2022-02-08T01:53:22+01:00",
          "tree_id": "ca8a9e19276ce452af1f124672e482633682b768",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/fdcbbbb2369346674ca9ca6945fac521f8abc089"
        },
        "date": 1644281860085,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 38999,
            "range": "± 237",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 892115,
            "range": "± 5389",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 14129482,
            "range": "± 189019",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 14149576,
            "range": "± 162303",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 48240487,
            "range": "± 2874043",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 42168,
            "range": "± 414",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 790985,
            "range": "± 2814",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 11064605,
            "range": "± 98871",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 11053235,
            "range": "± 49733",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 36650528,
            "range": "± 3181242",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "ef1b3be859c0c6e4cf7d502b7333cedd2973ee27",
          "message": "Add new tables",
          "timestamp": "2022-02-08T11:14:17+01:00",
          "tree_id": "e266e768b1447f61d6d10a0d7d8e2d00969548b7",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/ef1b3be859c0c6e4cf7d502b7333cedd2973ee27"
        },
        "date": 1644315559164,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 39029,
            "range": "± 161",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 954687,
            "range": "± 10549",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 17750265,
            "range": "± 228542",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 17913868,
            "range": "± 228492",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 166585639,
            "range": "± 4383295",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 40847,
            "range": "± 256",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 802529,
            "range": "± 9049",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 11426751,
            "range": "± 237696",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 11507163,
            "range": "± 193407",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 36349690,
            "range": "± 2295265",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "1b32e04bd5901fceb23871fd23dad0ff4a8e32e5",
          "message": "1e7 table",
          "timestamp": "2022-02-08T17:47:38+01:00",
          "tree_id": "c58056a302c9426cea8a309ce53112c5231ec67a",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/1b32e04bd5901fceb23871fd23dad0ff4a8e32e5"
        },
        "date": 1644339130376,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 39531,
            "range": "± 320",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 892462,
            "range": "± 7034",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 14259016,
            "range": "± 756549",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 14248665,
            "range": "± 537564",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 52402449,
            "range": "± 3789866",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 41723,
            "range": "± 365",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 793360,
            "range": "± 2314",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 11138371,
            "range": "± 347690",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 11125650,
            "range": "± 361228",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 39879111,
            "range": "± 4412816",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "8404180c00372628a1a8c6fdae65775aefa05bce",
          "message": "update tables",
          "timestamp": "2022-02-08T18:13:08+01:00",
          "tree_id": "2d59ea2008fb422316ef873a54633df6be89acd6",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/8404180c00372628a1a8c6fdae65775aefa05bce"
        },
        "date": 1644340650360,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 39032,
            "range": "± 290",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 889901,
            "range": "± 5032",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 14116944,
            "range": "± 153717",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 14176075,
            "range": "± 152243",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 47589256,
            "range": "± 2665106",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 41106,
            "range": "± 323",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 791120,
            "range": "± 14359",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 11049981,
            "range": "± 80983",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 11074581,
            "range": "± 137729",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 37496278,
            "range": "± 2564064",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "d063b0363a8bfa593f28092f879acdeeaa1719b3",
          "message": "N1e7 tools table",
          "timestamp": "2022-02-08T18:39:28+01:00",
          "tree_id": "e7133508123416e6b34288b6dc33c07568d6d36c",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/d063b0363a8bfa593f28092f879acdeeaa1719b3"
        },
        "date": 1644342238929,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 38893,
            "range": "± 363",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 897680,
            "range": "± 4689",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 14360103,
            "range": "± 217469",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 14386221,
            "range": "± 237986",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 48400834,
            "range": "± 3201355",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 41366,
            "range": "± 221",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 792413,
            "range": "± 4536",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 11082431,
            "range": "± 148406",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 11116073,
            "range": "± 213235",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 39773071,
            "range": "± 4168415",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "c8ac16a36b0888ce13542ccfc01159c252f12b22",
          "message": "Fix column names",
          "timestamp": "2022-02-08T18:39:59+01:00",
          "tree_id": "5709513bcc63b1cb6abbd57afbbd7584e046cd94",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/c8ac16a36b0888ce13542ccfc01159c252f12b22"
        },
        "date": 1644342272408,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 39206,
            "range": "± 268",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 900980,
            "range": "± 6616",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 14482352,
            "range": "± 369391",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 14483414,
            "range": "± 274438",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 49451843,
            "range": "± 1980871",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 40922,
            "range": "± 221",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 792250,
            "range": "± 2473",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 11176520,
            "range": "± 278913",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 11202539,
            "range": "± 223750",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 38251637,
            "range": "± 2579287",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ivanov@pesho.info",
            "name": "Pesho Ivanov",
            "username": "pesho-ivanov"
          },
          "committer": {
            "email": "ivanov@pesho.info",
            "name": "Pesho Ivanov",
            "username": "pesho-ivanov"
          },
          "distinct": true,
          "id": "68b76ed64742e3f0940a056ac8cd2f0c09dc0ce2",
          "message": "plot",
          "timestamp": "2022-02-08T19:12:35+01:00",
          "tree_id": "361b81c92a8d61d4c08860c5c348a4fbafb8a6db",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/68b76ed64742e3f0940a056ac8cd2f0c09dc0ce2"
        },
        "date": 1644344269165,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 38670,
            "range": "± 245",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 900040,
            "range": "± 9251",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 14436791,
            "range": "± 233940",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 14516619,
            "range": "± 308922",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 49560674,
            "range": "± 3630278",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 40990,
            "range": "± 151",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 788338,
            "range": "± 3325",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 11222096,
            "range": "± 333509",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 11264716,
            "range": "± 298896",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 39096631,
            "range": "± 4313079",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "9de048b806a5705a1dd727c01af22501e42328f7",
          "message": "ADD DIAGONAL MAP OFFSET\n\nUsually a bit faster. 10% slower at worst",
          "timestamp": "2022-02-09T00:04:58+01:00",
          "tree_id": "f7262077a204cf464585ed28d12689c038bb7a4c",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/9de048b806a5705a1dd727c01af22501e42328f7"
        },
        "date": 1644361806313,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 39518,
            "range": "± 210",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 944254,
            "range": "± 12933",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 17825924,
            "range": "± 229484",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 17925225,
            "range": "± 152706",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 165354389,
            "range": "± 1921961",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 41686,
            "range": "± 255",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 805365,
            "range": "± 11998",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 11451055,
            "range": "± 65591",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 11474475,
            "range": "± 80787",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 35027203,
            "range": "± 2148549",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "40e7a3fcfc1c77cc1fe379fa9b8f177757bf308b",
          "message": "ADD DO_NOT_SAVE_GREEDY_MATCHES for walking matches faster and using less memory",
          "timestamp": "2022-02-09T00:06:28+01:00",
          "tree_id": "0aa0b25e732aea2177453f825dcc1d9688c0f3bd",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/40e7a3fcfc1c77cc1fe379fa9b8f177757bf308b"
        },
        "date": 1644361870579,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 38689,
            "range": "± 183",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 924701,
            "range": "± 4276",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 14892119,
            "range": "± 25594",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 14931128,
            "range": "± 25699",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 43423598,
            "range": "± 1248110",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 39160,
            "range": "± 140",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 802049,
            "range": "± 2482",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 11262814,
            "range": "± 13142",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 11284790,
            "range": "± 24303",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 31899159,
            "range": "± 574522",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "bbbf24cfed32156777ee80160bd4bad8ef9c317b",
          "message": "ADD SLIDING_WINDOW_MATCHES: smaller qgram hashmaps at no/small time cost",
          "timestamp": "2022-02-09T00:50:48+01:00",
          "tree_id": "d008ca1ac0af57d45ea7896f5128094b6a28969d",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/bbbf24cfed32156777ee80160bd4bad8ef9c317b"
        },
        "date": 1644364540345,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 39083,
            "range": "± 269",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 914688,
            "range": "± 5992",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 14788812,
            "range": "± 206593",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 14825395,
            "range": "± 150100",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 47120170,
            "range": "± 2098971",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 38407,
            "range": "± 143",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 796223,
            "range": "± 2527",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 11267018,
            "range": "± 128040",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 11317775,
            "range": "± 241848",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 37351369,
            "range": "± 2048475",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "62a9e42117b9b19527268ee3b4b862ab559d04f5",
          "message": "Do not comment on slow benches",
          "timestamp": "2022-02-09T00:53:21+01:00",
          "tree_id": "8e8da1a4fa8bfc72a440a83ff29f0e782589a49f",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/62a9e42117b9b19527268ee3b4b862ab559d04f5"
        },
        "date": 1644364670670,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 39231,
            "range": "± 410",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 915762,
            "range": "± 6097",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 14716299,
            "range": "± 297436",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 14854211,
            "range": "± 570613",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 47037796,
            "range": "± 2921731",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 39179,
            "range": "± 176",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 794731,
            "range": "± 2255",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 11202108,
            "range": "± 145303",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 11188138,
            "range": "± 76709",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 36095397,
            "range": "± 4999223",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "b90801fcdca98b0a347955242564a350b5fbbc1b",
          "message": "Disable broken test",
          "timestamp": "2022-02-09T00:58:24+01:00",
          "tree_id": "59406bd65eacc1f896322dbee0a121a2c4a6b93c",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/b90801fcdca98b0a347955242564a350b5fbbc1b"
        },
        "date": 1644365009482,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 38625,
            "range": "± 175",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 964621,
            "range": "± 11474",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 18577617,
            "range": "± 416536",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 18723856,
            "range": "± 233828",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 164446839,
            "range": "± 1133991",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 39946,
            "range": "± 275",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 823595,
            "range": "± 11932",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 11946660,
            "range": "± 278570",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 12013674,
            "range": "± 346223",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 35082588,
            "range": "± 1965032",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "b1ade1ebacefbaccf06578f73ebbc6d75c7f0580",
          "message": "disable more test",
          "timestamp": "2022-02-09T01:06:39+01:00",
          "tree_id": "efce5950eda568b72ae4a08f25aba98b204dba97",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/b1ade1ebacefbaccf06578f73ebbc6d75c7f0580"
        },
        "date": 1644365466029,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 38949,
            "range": "± 328",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 914057,
            "range": "± 6455",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 14746176,
            "range": "± 179935",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 14932918,
            "range": "± 394416",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 45777054,
            "range": "± 4613375",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 38274,
            "range": "± 660",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 798006,
            "range": "± 2559",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 11300339,
            "range": "± 177981",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 11344469,
            "range": "± 212724",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 34099759,
            "range": "± 4688030",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "2fc393bdd3a78a45b7edcc32274293a51f928431",
          "message": "Only check match consistency for m > 0",
          "timestamp": "2022-02-09T02:03:59+01:00",
          "tree_id": "712986fe66f438d849a1b908def3dc6f3c1860ba",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/2fc393bdd3a78a45b7edcc32274293a51f928431"
        },
        "date": 1644368913516,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 38850,
            "range": "± 231",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 915187,
            "range": "± 5173",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 14856355,
            "range": "± 203509",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 14894653,
            "range": "± 296900",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 46200359,
            "range": "± 2979605",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 38114,
            "range": "± 244",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 798223,
            "range": "± 2605",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 11209194,
            "range": "± 78450",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 11203759,
            "range": "± 76251",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 33963323,
            "range": "± 3206592",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ivanov@pesho.info",
            "name": "Pesho Ivanov",
            "username": "pesho-ivanov"
          },
          "committer": {
            "email": "ivanov@pesho.info",
            "name": "Pesho Ivanov",
            "username": "pesho-ivanov"
          },
          "distinct": true,
          "id": "fd6dc0cf23ac5430c95bf4781d12ecfdf0746c99",
          "message": "update",
          "timestamp": "2022-02-09T11:19:08+01:00",
          "tree_id": "b5d95398fb1e7c32ce79ed7afe91f5fa432a753e",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/fd6dc0cf23ac5430c95bf4781d12ecfdf0746c99"
        },
        "date": 1644402249546,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 38242,
            "range": "± 267",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 973522,
            "range": "± 16808",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 18715686,
            "range": "± 226360",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 18849908,
            "range": "± 248310",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 163987322,
            "range": "± 742960",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 39435,
            "range": "± 471",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 815054,
            "range": "± 8552",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 11796210,
            "range": "± 125680",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 11781390,
            "range": "± 94295",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 34555922,
            "range": "± 1947156",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "3867f28804117fb3b58442b304cc1aefceedc3da",
          "message": "New tables",
          "timestamp": "2022-02-09T11:34:24+01:00",
          "tree_id": "ceadf1da8a4df90675923caef843887e1bc9dffb",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/3867f28804117fb3b58442b304cc1aefceedc3da"
        },
        "date": 1644403133599,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 33879,
            "range": "± 658",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 805701,
            "range": "± 32411",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 13202969,
            "range": "± 626559",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 13149227,
            "range": "± 399767",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 42696221,
            "range": "± 2011077",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 34317,
            "range": "± 1041",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 700209,
            "range": "± 8912",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 9952064,
            "range": "± 294177",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 9928959,
            "range": "± 239208",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 32005782,
            "range": "± 1574611",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ivanov@pesho.info",
            "name": "Pesho Ivanov",
            "username": "pesho-ivanov"
          },
          "committer": {
            "email": "ivanov@pesho.info",
            "name": "Pesho Ivanov",
            "username": "pesho-ivanov"
          },
          "distinct": true,
          "id": "e1c4b81f1a358206ad83ab9035be396a589246b0",
          "message": "k vs band",
          "timestamp": "2022-02-09T12:38:39+01:00",
          "tree_id": "a3fcab6b9054a8e04327061f14f920c209e9ca93",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/e1c4b81f1a358206ad83ab9035be396a589246b0"
        },
        "date": 1644406976180,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 38783,
            "range": "± 321",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 914049,
            "range": "± 4946",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 14811914,
            "range": "± 94466",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 14835004,
            "range": "± 120908",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 45902044,
            "range": "± 1086740",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 38341,
            "range": "± 171",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 795856,
            "range": "± 2979",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 11203723,
            "range": "± 51003",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 11216093,
            "range": "± 59725",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 34494021,
            "range": "± 2238909",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "13e7fce38a6184a037e0b2165322c0891e0636be",
          "message": "small bugfixes",
          "timestamp": "2022-02-10T12:51:58+01:00",
          "tree_id": "ea8ec949a09e399db5d2f77c92d8637f8ed14670",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/13e7fce38a6184a037e0b2165322c0891e0636be"
        },
        "date": 1644494184547,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 39845,
            "range": "± 917",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 925787,
            "range": "± 7904",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 15364591,
            "range": "± 405436",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 15403516,
            "range": "± 442101",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 49825687,
            "range": "± 1877017",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 38350,
            "range": "± 337",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 796597,
            "range": "± 3044",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 11396409,
            "range": "± 409722",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 11195327,
            "range": "± 51983",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 34721470,
            "range": "± 2369034",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "3d67354c8757a6b2d512d82ae59b38ea1fc6656e",
          "message": "Fix test",
          "timestamp": "2022-02-10T13:04:02+01:00",
          "tree_id": "bef0b84499857a951ad8d822c521b4588620d29a",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/3d67354c8757a6b2d512d82ae59b38ea1fc6656e"
        },
        "date": 1644494945989,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 39319,
            "range": "± 237",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1004042,
            "range": "± 18149",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 18826667,
            "range": "± 376473",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 19067182,
            "range": "± 361103",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 164666889,
            "range": "± 1407606",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 39535,
            "range": "± 675",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 812915,
            "range": "± 12761",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 11794891,
            "range": "± 114383",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 11860190,
            "range": "± 260329",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 34713825,
            "range": "± 2074810",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "c21094ed7d137b058a4acf40422e2dd9bf4144f3",
          "message": "Snakemake: Report exit status in table",
          "timestamp": "2022-02-10T16:06:42+01:00",
          "tree_id": "1da84d274f434859fe0d8b411d2dc5c3ac9a35bd",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/c21094ed7d137b058a4acf40422e2dd9bf4144f3"
        },
        "date": 1644505897336,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 39951,
            "range": "± 439",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 921247,
            "range": "± 6712",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 15832005,
            "range": "± 402067",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 15813668,
            "range": "± 372376",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 53734539,
            "range": "± 1690276",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 39982,
            "range": "± 257",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 793792,
            "range": "± 3876",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 11728218,
            "range": "± 327366",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 11683208,
            "range": "± 270346",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 42015313,
            "range": "± 2256092",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "c1d2732c00b887b6215da7f768407bb49671faa1",
          "message": "Remove block-aligner dependency for generating random sequences",
          "timestamp": "2022-02-11T17:48:58+01:00",
          "tree_id": "4cfb64ade5641c5ed141b7ab81bc202e51e06ae4",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/c1d2732c00b887b6215da7f768407bb49671faa1"
        },
        "date": 1644598446501,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 38103,
            "range": "± 221",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1054744,
            "range": "± 16402",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 20679371,
            "range": "± 184064",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 20707500,
            "range": "± 135860",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 158983534,
            "range": "± 1060022",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 38069,
            "range": "± 290",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 847808,
            "range": "± 8816",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 13004179,
            "range": "± 193793",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 12957229,
            "range": "± 170792",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 34725375,
            "range": "± 1276061",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "8bb9dc8653c7a0dfa210bdb7bbbd2b5bffea206b",
          "message": "Merge matches and sorting benchmarks into one file to remove aho-corasick dependency",
          "timestamp": "2022-02-14T10:41:09+01:00",
          "tree_id": "0d151a21574a3afd6920869868a0eaf95c67b34e",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/8bb9dc8653c7a0dfa210bdb7bbbd2b5bffea206b"
        },
        "date": 1644831937185,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 36321,
            "range": "± 270",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 968547,
            "range": "± 4054",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 16333078,
            "range": "± 524653",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 16463489,
            "range": "± 534424",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 48059244,
            "range": "± 2501075",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 36713,
            "range": "± 147",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 832580,
            "range": "± 2263",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 12269044,
            "range": "± 252985",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 12235223,
            "range": "± 162754",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 35027770,
            "range": "± 2904113",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "39534df958a74667e9e4955ae647999143b838f1",
          "message": "Simplify BruteForceContour[s] and use SmallVec<[Pos; 2]>",
          "timestamp": "2022-02-14T10:49:29+01:00",
          "tree_id": "335bce9647e1a2c07b89dc63a15f02e56a46d78b",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/39534df958a74667e9e4955ae647999143b838f1"
        },
        "date": 1644832467487,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 37647,
            "range": "± 141",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1034663,
            "range": "± 13582",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 19836067,
            "range": "± 75801",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 19862550,
            "range": "± 110550",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 155081185,
            "range": "± 1359370",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 38671,
            "range": "± 206",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 855342,
            "range": "± 12450",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 12892819,
            "range": "± 37795",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 12890885,
            "range": "± 39662",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 31414297,
            "range": "± 379761",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "1af4682cb750b0e096f0c3bc0783c859ebde020c",
          "message": "Use SmallVec for CentralContour",
          "timestamp": "2022-02-14T10:50:25+01:00",
          "tree_id": "00c96362cf2ee98e8aeea79ab59b32c9ec4ed50e",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/1af4682cb750b0e096f0c3bc0783c859ebde020c"
        },
        "date": 1644832482501,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 36488,
            "range": "± 285",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 980601,
            "range": "± 6567",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 16924415,
            "range": "± 573454",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 16931615,
            "range": "± 454704",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 49686459,
            "range": "± 1454499",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 37876,
            "range": "± 1385",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 896632,
            "range": "± 3735",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 13499940,
            "range": "± 453002",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 13394648,
            "range": "± 275903",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 39859599,
            "range": "± 3017047",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "6b382889e4fa8847cb6b6ca6078582b783a841b9",
          "message": "DELETE LOG_QUERY_CONTOUR AND SET_CONTOUR\n\nThese are much more complicated than bruteforce and central contour, and\nnever faster than both of those.",
          "timestamp": "2022-02-14T11:26:59+01:00",
          "tree_id": "4ab2b405630cf9871a06bc4a218a5fc7d6ddada1",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/6b382889e4fa8847cb6b6ca6078582b783a841b9"
        },
        "date": 1644834735312,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 38291,
            "range": "± 113",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 887121,
            "range": "± 5013",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 13018484,
            "range": "± 170123",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 13038398,
            "range": "± 208486",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 38343457,
            "range": "± 3033900",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 33193,
            "range": "± 93",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 809770,
            "range": "± 2617",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 12957372,
            "range": "± 255776",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 12990872,
            "range": "± 363668",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 33831798,
            "range": "± 2460417",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "fe4237cbd3ef7c9cfebc302e73f2a6c111a2a5d0",
          "message": "DELETE NAIVE_CONTOURS\n\nIt's never faster than HintContours",
          "timestamp": "2022-02-14T11:43:33+01:00",
          "tree_id": "3351f7757faa8efc1b0172a2d0ab3b7bbb0b9f15",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/fe4237cbd3ef7c9cfebc302e73f2a6c111a2a5d0"
        },
        "date": 1644835644488,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 33372,
            "range": "± 1243",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 778254,
            "range": "± 32543",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 11463579,
            "range": "± 201366",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 11468549,
            "range": "± 164377",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 31319890,
            "range": "± 2909671",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 29258,
            "range": "± 2375",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 717307,
            "range": "± 21532",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 11313409,
            "range": "± 231152",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 11337790,
            "range": "± 227586",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 27885540,
            "range": "± 2007121",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "222b748e7d0d3c6e86a7e666c503f787323b0185",
          "message": "Remove util.rs and move content to seeds.rs and lib.rs",
          "timestamp": "2022-02-14T12:34:35+01:00",
          "tree_id": "63038bb37506dcda238a295d4a176ef4b6fe0bb5",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/222b748e7d0d3c6e86a7e666c503f787323b0185"
        },
        "date": 1644838826059,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 33649,
            "range": "± 1333",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 780669,
            "range": "± 33983",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 11421232,
            "range": "± 110945",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 11338709,
            "range": "± 274359",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 31078978,
            "range": "± 2742182",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 28756,
            "range": "± 914",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 715318,
            "range": "± 13093",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 11280128,
            "range": "± 236928",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 11326178,
            "range": "± 143518",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 28487369,
            "range": "± 3686452",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "4731f281742b81846805052146d2da7fd2076bbe",
          "message": "Move contours/traits.rs into contours.rs itself",
          "timestamp": "2022-02-14T12:59:13+01:00",
          "tree_id": "a38de7143b11d3a7d9aefb0ba40eedb4a45efd21",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/4731f281742b81846805052146d2da7fd2076bbe"
        },
        "date": 1644840355277,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 38981,
            "range": "± 193",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 917430,
            "range": "± 10138",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 13672610,
            "range": "± 100864",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 13665793,
            "range": "± 94357",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 36321289,
            "range": "± 1398572",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 34591,
            "range": "± 215",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 844021,
            "range": "± 10845",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 14082887,
            "range": "± 152028",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 14117911,
            "range": "± 119654",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 32262835,
            "range": "± 959714",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "cb9bd149831f9ebc799d7b4d666eebff82da0f79",
          "message": "INLINE IMPLICIT_GRAPH (Pos, Parent, DiagonalMap) and delete the file",
          "timestamp": "2022-02-14T14:07:57+01:00",
          "tree_id": "5b5b9ec67be1de53793358647b85cccc3382c10a",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/cb9bd149831f9ebc799d7b4d666eebff82da0f79"
        },
        "date": 1644844336452,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 33617,
            "range": "± 718",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 784915,
            "range": "± 14485",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 11506665,
            "range": "± 87379",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 11515935,
            "range": "± 70678",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 32064460,
            "range": "± 2094684",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 29473,
            "range": "± 163",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 717513,
            "range": "± 5768",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 11453781,
            "range": "± 95546",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 11460121,
            "range": "± 68111",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 28807040,
            "range": "± 1313858",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "5e3ed74a55ae0cad2ca137f01814da5ca1701db9",
          "message": "Make Heuristic{Params,Stats} value plain ints instead of Optionals",
          "timestamp": "2022-02-14T14:54:00+01:00",
          "tree_id": "19783cdc572c5fd129d3e5e6840495a57d72e048",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/5e3ed74a55ae0cad2ca137f01814da5ca1701db9"
        },
        "date": 1644847115059,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 37936,
            "range": "± 194",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 888765,
            "range": "± 4225",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 13131055,
            "range": "± 457444",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 13061810,
            "range": "± 197093",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 38165719,
            "range": "± 2073728",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 33450,
            "range": "± 160",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 812231,
            "range": "± 6257",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 13104666,
            "range": "± 383802",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 12919924,
            "range": "± 138136",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 33515746,
            "range": "± 1815748",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "ade8ec9b141facb9cd4b6ef80efcdda36369e275",
          "message": "Delete duplicate tests",
          "timestamp": "2022-02-14T17:55:42+01:00",
          "tree_id": "af2de88b21b7187be3aa4686979d883707a46f74",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/ade8ec9b141facb9cd4b6ef80efcdda36369e275"
        },
        "date": 1644860996578,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 38132,
            "range": "± 359",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 889681,
            "range": "± 8022",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 13218051,
            "range": "± 650585",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 13101581,
            "range": "± 287117",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 39370974,
            "range": "± 3826037",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 33580,
            "range": "± 197",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 810700,
            "range": "± 36586",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 13043189,
            "range": "± 269491",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 13153660,
            "range": "± 429459",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 34333627,
            "range": "± 5957058",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "301d873b076b7bfa0e47dc462ed471eb6b54c286",
          "message": "Fix detection of neighbouring matches preventing pruning",
          "timestamp": "2022-02-14T19:23:21+01:00",
          "tree_id": "9774b3d8093f915977b19af6024feb51d58f2cbe",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/301d873b076b7bfa0e47dc462ed471eb6b54c286"
        },
        "date": 1644863371466,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 37662,
            "range": "± 273",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 891110,
            "range": "± 9100",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 13578290,
            "range": "± 62661",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000_prune_less",
            "value": 13583260,
            "range": "± 50559",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 34816457,
            "range": "± 1096940",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 33717,
            "range": "± 257",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 821479,
            "range": "± 6993",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 13792181,
            "range": "± 119566",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000_prune_less",
            "value": 13813991,
            "range": "± 92763",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 32371383,
            "range": "± 2208156",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "255b52cde405dc7e776b059ea061023ca866362b",
          "message": "REMOVE PRUNE_FRACTION\n\nAlways set to 1 in experiments anyway, since it doesn't provide too much\nbenefit.",
          "timestamp": "2022-02-15T17:46:44+01:00",
          "tree_id": "72f27f1236c32f062134df0b52a4e9bc40e98f1a",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/255b52cde405dc7e776b059ea061023ca866362b"
        },
        "date": 1644943868438,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 31744,
            "range": "± 1128",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 750126,
            "range": "± 31635",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 11172949,
            "range": "± 378851",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 29421808,
            "range": "± 1471031",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 28446,
            "range": "± 2585",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 690616,
            "range": "± 18069",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 11172829,
            "range": "± 187136",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 26148435,
            "range": "± 2843652",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "dacdfea3bbc3254b9bbae6916dea6d5798e3659e",
          "message": "Fix start_of_seed error",
          "timestamp": "2022-02-16T15:23:03+01:00",
          "tree_id": "e0c8691bfa3b81a4cbb3e40c2338a3e94fc0b627",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/dacdfea3bbc3254b9bbae6916dea6d5798e3659e"
        },
        "date": 1645021634607,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 37306,
            "range": "± 465",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 853630,
            "range": "± 6587",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 13087805,
            "range": "± 934253",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 42470962,
            "range": "± 2591717",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 33180,
            "range": "± 569",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 772211,
            "range": "± 2822",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 13707055,
            "range": "± 755034",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 37752280,
            "range": "± 5745979",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "8c63d7e2586cfbb8277ec298ef26a310706b26dc",
          "message": "Track SEED_COST in A* state, for faster match detection",
          "timestamp": "2022-02-16T16:21:20+01:00",
          "tree_id": "304c82d33e53685b4df651f8cc2ab46486369bb0",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/8c63d7e2586cfbb8277ec298ef26a310706b26dc"
        },
        "date": 1645031855199,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 37889,
            "range": "± 556",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 867821,
            "range": "± 6008",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 13171978,
            "range": "± 246535",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 38185860,
            "range": "± 2211377",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 33760,
            "range": "± 184",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 784590,
            "range": "± 10577",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 12955065,
            "range": "± 341626",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 34071012,
            "range": "± 2439198",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "6d998931de1559ceac7e9a1c2ee2e22e47eba6c6",
          "message": "Replace start_of_seed by seed_at vector",
          "timestamp": "2022-02-16T18:57:39+01:00",
          "tree_id": "9b5f7a10bebed416c1590443eafda296be5e099b",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/6d998931de1559ceac7e9a1c2ee2e22e47eba6c6"
        },
        "date": 1645034464994,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 47952,
            "range": "± 194",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 44189,
            "range": "± 308",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "8bb7b97c129a6165468dcbd139b1db3d379d652b",
          "message": "Replace start_of_seed by seed_at vector",
          "timestamp": "2022-02-16T19:27:15+01:00",
          "tree_id": "104848ce30122940a980b1add54512d6b4429cce",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/8bb7b97c129a6165468dcbd139b1db3d379d652b"
        },
        "date": 1645036265937,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 38609,
            "range": "± 327",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 883825,
            "range": "± 9264",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 13479232,
            "range": "± 144313",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 34816404,
            "range": "± 1158984",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 34482,
            "range": "± 343",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 826033,
            "range": "± 17484",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 13834201,
            "range": "± 157491",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 30594105,
            "range": "± 1031327",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "c6fd8a7d63ee3bc7f0ac38945f4a0bb7cd0eb81d",
          "message": "Rename is_start_of_seed => is_seed_start_or_end\n\nFor UnorderedHeuristic, pruning happens at the end of matches instead of\nat the start, so the consistency requirements and when prune() is called changes.\nThis is especially needed since UnorderedHeuristic doesn't cover the\nentire length with seeds.",
          "timestamp": "2022-02-16T19:33:32+01:00",
          "tree_id": "a2e8b54d284a02751acae314d5a3026bb0108bd4",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/c6fd8a7d63ee3bc7f0ac38945f4a0bb7cd0eb81d"
        },
        "date": 1645036745953,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 33192,
            "range": "± 230",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 760639,
            "range": "± 9141",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 11416262,
            "range": "± 112114",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 31886522,
            "range": "± 3610176",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 29634,
            "range": "± 228",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 692898,
            "range": "± 2862",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 11286872,
            "range": "± 111278",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 29235280,
            "range": "± 2959575",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "82d73e44bb57c47b8b4ec9412af68dc59a54b190",
          "message": "ADD UNORDERED_MATCHES",
          "timestamp": "2022-02-16T21:43:45+01:00",
          "tree_id": "456a97daadcdad588ddacd54f748882550533eae",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/82d73e44bb57c47b8b4ec9412af68dc59a54b190"
        },
        "date": 1645044510284,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 44237,
            "range": "± 7171",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1038739,
            "range": "± 176811",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 14930757,
            "range": "± 1491011",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 44090639,
            "range": "± 5086648",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 39781,
            "range": "± 3479",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 942886,
            "range": "± 130535",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 15045686,
            "range": "± 1606625",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 37236489,
            "range": "± 4364372",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "82d73e44bb57c47b8b4ec9412af68dc59a54b190",
          "message": "ADD UNORDERED_MATCHES",
          "timestamp": "2022-02-16T21:43:45+01:00",
          "tree_id": "456a97daadcdad588ddacd54f748882550533eae",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/82d73e44bb57c47b8b4ec9412af68dc59a54b190"
        },
        "date": 1645044986213,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 39229,
            "range": "± 180",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 889570,
            "range": "± 11595",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 13638245,
            "range": "± 61284",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 34382440,
            "range": "± 1692126",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 34882,
            "range": "± 258",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 819944,
            "range": "± 11399",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 13801895,
            "range": "± 144709",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 31267008,
            "range": "± 2179149",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "dfa64e5f3072b88e19d5bb7ee66bf51889814ccd",
          "message": "Change Seed::has_matches to seed_cost, for potentials with inexact matches",
          "timestamp": "2022-02-17T23:14:19+01:00",
          "tree_id": "52dcbac9134b8cefbc0bd4e888ccbef219bfdfaf",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/dfa64e5f3072b88e19d5bb7ee66bf51889814ccd"
        },
        "date": 1645136325207,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 37008,
            "range": "± 304",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 870652,
            "range": "± 28969",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 12598818,
            "range": "± 59067",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 31007029,
            "range": "± 1338059",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 32231,
            "range": "± 97",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 784889,
            "range": "± 2949",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 12326482,
            "range": "± 43699",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 27019113,
            "range": "± 873374",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "8762e795260aa07cbc9db8b5be5e21fd73030c65",
          "message": "Split ordered matchers out of matches.rs",
          "timestamp": "2022-02-18T11:32:09+01:00",
          "tree_id": "b99a0df513c9fe6d54e3abdcd105b74ba793a8f7",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/8762e795260aa07cbc9db8b5be5e21fd73030c65"
        },
        "date": 1645180612504,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 37524,
            "range": "± 507",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 902983,
            "range": "± 9105",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 13458605,
            "range": "± 140782",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 34354170,
            "range": "± 730539",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 33808,
            "range": "± 277",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 822344,
            "range": "± 13482",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 13597777,
            "range": "± 136568",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 29904290,
            "range": "± 2086803",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "b9ff900bcd32c485f8e2769e6311f459bcbcd515",
          "message": "Add missing onordered_filters file",
          "timestamp": "2022-02-18T12:04:18+01:00",
          "tree_id": "7ca96eebe4a3a301419b6446c166749d37efe770",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/b9ff900bcd32c485f8e2769e6311f459bcbcd515"
        },
        "date": 1645182504540,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 36748,
            "range": "± 302",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 870849,
            "range": "± 5855",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 12975325,
            "range": "± 478510",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 36797822,
            "range": "± 1631126",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 32742,
            "range": "± 302",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 789728,
            "range": "± 2626",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 12741653,
            "range": "± 465923",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 33576698,
            "range": "± 2433226",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "0038353fb47bbec0a2b0d25f20aa18db6844580a",
          "message": "Replace unordered matches by a single generic implementation",
          "timestamp": "2022-02-18T13:44:15+01:00",
          "tree_id": "98e681382b15625c3df5769357d91f00a254e257",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/0038353fb47bbec0a2b0d25f20aa18db6844580a"
        },
        "date": 1645188481275,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 32787,
            "range": "± 324",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 766809,
            "range": "± 6148",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 11227903,
            "range": "± 95363",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 29469121,
            "range": "± 2584267",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 28953,
            "range": "± 188",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 696988,
            "range": "± 9298",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 11029221,
            "range": "± 139961",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 25856921,
            "range": "± 1317227",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "8129fdd2b4036fb039fbbad50dc1d02fdbc5e240",
          "message": "add link to ONP data",
          "timestamp": "2022-02-27T10:56:07+01:00",
          "tree_id": "afb0c2f69084d42598823d373f6c612fb2773825",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/8129fdd2b4036fb039fbbad50dc1d02fdbc5e240"
        },
        "date": 1645956004272,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 36839,
            "range": "± 211",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 872121,
            "range": "± 6648",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 12654799,
            "range": "± 43492",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 31553920,
            "range": "± 1028276",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 32064,
            "range": "± 147",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 777750,
            "range": "± 2206",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 12250563,
            "range": "± 177345",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 27560708,
            "range": "± 1231001",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "432e1cd73da52ab5e79eb80500011c2dacbf1172",
          "message": "Add ONT reads commands to snakefile",
          "timestamp": "2022-03-30T20:31:59+02:00",
          "tree_id": "820cc6441d2d575009364ccd8554e301de721d63",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/432e1cd73da52ab5e79eb80500011c2dacbf1172"
        },
        "date": 1648665390767,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 37262,
            "range": "± 301",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 875219,
            "range": "± 13637",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 13500301,
            "range": "± 168055",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 35923958,
            "range": "± 3376508",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 32871,
            "range": "± 253",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 807924,
            "range": "± 11648",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 13838456,
            "range": "± 288067",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 31952062,
            "range": "± 1811132",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "6615082695c56a1efcda710e73737154252c81d5",
          "message": "Extract path reconstruction from A* body",
          "timestamp": "2022-04-08T14:10:55+02:00",
          "tree_id": "8e913f0213125a23f7fdd869981940bc171ea23b",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/6615082695c56a1efcda710e73737154252c81d5"
        },
        "date": 1649420157343,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 47686,
            "range": "± 7671",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1079062,
            "range": "± 307476",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 16195800,
            "range": "± 1771708",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 46402581,
            "range": "± 7264625",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 43290,
            "range": "± 11794",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 975523,
            "range": "± 266279",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 15684836,
            "range": "± 1585688",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 38899947,
            "range": "± 6959325",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "fbf261b40bbce313f450bc8f8703aec6578ba84b",
          "message": "Add heuristic ops counters",
          "timestamp": "2022-04-08T14:58:57+02:00",
          "tree_id": "9d9f5f4a40f439464ed8e6ee22f843d5050ac8eb",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/fbf261b40bbce313f450bc8f8703aec6578ba84b"
        },
        "date": 1649422963423,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 36818,
            "range": "± 4659",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 866571,
            "range": "± 6999",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 13230084,
            "range": "± 920511",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 34114110,
            "range": "± 994576",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 34472,
            "range": "± 259",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 799213,
            "range": "± 8837",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 13658851,
            "range": "± 329104",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 29656462,
            "range": "± 903494",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "17ff11890cc592ebe914bc396504d7a7a052eaa3",
          "message": "fix ops counting",
          "timestamp": "2022-04-08T17:16:08+02:00",
          "tree_id": "8f46133dbdc6327729f8bcccad4ee77bd0841201",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/17ff11890cc592ebe914bc396504d7a7a052eaa3"
        },
        "date": 1649431181185,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 36043,
            "range": "± 578",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 845400,
            "range": "± 5572",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 12883007,
            "range": "± 482322",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 37600257,
            "range": "± 2814470",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 31853,
            "range": "± 153",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 757389,
            "range": "± 3479",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 12352260,
            "range": "± 303737",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 32173396,
            "range": "± 2587906",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "726ad1444d1150f4f5058afd44f1c08205a3c3b2",
          "message": "Add script to compute unordered heuristic stats",
          "timestamp": "2022-04-08T18:18:16+02:00",
          "tree_id": "cc628c74919417833d766daeb6bedf6ebabbe529",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/726ad1444d1150f4f5058afd44f1c08205a3c3b2"
        },
        "date": 1649434964454,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 31898,
            "range": "± 1277",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 745718,
            "range": "± 6376",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 12043086,
            "range": "± 606917",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 33600344,
            "range": "± 5012785",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 28041,
            "range": "± 399",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 667617,
            "range": "± 6188",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 10912587,
            "range": "± 509122",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 28396691,
            "range": "± 5761362",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "0ed23535af95525cc0ef5c1af59f8945dde9561e",
          "message": "update evals",
          "timestamp": "2022-04-08T20:58:41+02:00",
          "tree_id": "63efc2394c74216e113e1ed98e644583ec319f9c",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/0ed23535af95525cc0ef5c1af59f8945dde9561e"
        },
        "date": 1649444531235,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 36448,
            "range": "± 469",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 848785,
            "range": "± 4822",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 12432841,
            "range": "± 50333",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 32315532,
            "range": "± 3597678",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 31739,
            "range": "± 159",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 756545,
            "range": "± 3095",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 12174968,
            "range": "± 220508",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 27498219,
            "range": "± 3132159",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "b4a0af3adef0abba86bc65837fb59394945f54a3",
          "message": "move best-param-finding to run_tool",
          "timestamp": "2022-04-18T01:01:51+02:00",
          "tree_id": "dc1c4a05331ea356aab24eb6841849b36d0a494e",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/b4a0af3adef0abba86bc65837fb59394945f54a3"
        },
        "date": 1650236725224,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 37194,
            "range": "± 241",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 860206,
            "range": "± 7060",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 11106208,
            "range": "± 1467094",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 28060660,
            "range": "± 1852575",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 28206,
            "range": "± 108",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 677823,
            "range": "± 2600",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 10891390,
            "range": "± 63835",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 24261585,
            "range": "± 2946425",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "a12954f7de96d80ad151a71598c0a21bb52dc5f9",
          "message": "Update N1e7 tables",
          "timestamp": "2022-04-18T08:15:45+02:00",
          "tree_id": "4945095823fb1147bbd1fe795f4d9acac8f7f95f",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/a12954f7de96d80ad151a71598c0a21bb52dc5f9"
        },
        "date": 1650262786374,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 36814,
            "range": "± 240",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 863910,
            "range": "± 7054",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 12558615,
            "range": "± 87110",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 31612060,
            "range": "± 1852437",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 32346,
            "range": "± 199",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 772029,
            "range": "± 2642",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 12294411,
            "range": "± 54152",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 26887504,
            "range": "± 1719366",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "f11d41787d5a8772259fd535c7943f94cb358b71",
          "message": "remove bad param runs from tools tables",
          "timestamp": "2022-04-18T08:46:07+02:00",
          "tree_id": "823f565cbea25b7c2cb65aa447732c5cdaaf3ef4",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/f11d41787d5a8772259fd535c7943f94cb358b71"
        },
        "date": 1650264584214,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 37283,
            "range": "± 213",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 864360,
            "range": "± 7258",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 12666939,
            "range": "± 300412",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 35174462,
            "range": "± 3241745",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 32655,
            "range": "± 119",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 772339,
            "range": "± 3471",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 12378901,
            "range": "± 205233",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 28638870,
            "range": "± 3427355",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "2a659e0c31086e5ba0890c62664dcb5a54cad558",
          "message": "Update evals: cleanup; time+memory plots",
          "timestamp": "2022-04-18T12:48:54+02:00",
          "tree_id": "f4a9d9cc525b3441439cdeee6a0471ceaf561b75",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/2a659e0c31086e5ba0890c62664dcb5a54cad558"
        },
        "date": 1650279160024,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 37189,
            "range": "± 240",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 863115,
            "range": "± 8399",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 12521707,
            "range": "± 64562",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 31665890,
            "range": "± 2872596",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 32391,
            "range": "± 136",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 771263,
            "range": "± 2915",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 12282529,
            "range": "± 83061",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 26990363,
            "range": "± 1441321",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "78e4ebf5a646ac738fc6588f8f03cfa41c792893",
          "message": "update rendering and colors",
          "timestamp": "2022-04-18T12:49:57+02:00",
          "tree_id": "570cfb4262da891a4ec1039c8ec3caffdd5548f4",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/78e4ebf5a646ac738fc6588f8f03cfa41c792893"
        },
        "date": 1650279206934,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 37258,
            "range": "± 352",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 862412,
            "range": "± 4195",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 12526489,
            "range": "± 30140",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 30771224,
            "range": "± 528136",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 32567,
            "range": "± 99",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 770270,
            "range": "± 3084",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 12269171,
            "range": "± 22957",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 26468463,
            "range": "± 823607",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ivanov@pesho.info",
            "name": "Pesho Ivanov",
            "username": "pesho-ivanov"
          },
          "committer": {
            "email": "ivanov@pesho.info",
            "name": "Pesho Ivanov",
            "username": "pesho-ivanov"
          },
          "distinct": true,
          "id": "31a48dd1b9c6181bc8ebe435b466427c2dc37510",
          "message": "correct fitting",
          "timestamp": "2022-04-18T15:24:46+02:00",
          "tree_id": "23026af43b5e4d61e841367948d3034fab0d2589",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/31a48dd1b9c6181bc8ebe435b466427c2dc37510"
        },
        "date": 1650288492321,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 37049,
            "range": "± 524",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 862116,
            "range": "± 8461",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 12791831,
            "range": "± 323667",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 34810415,
            "range": "± 1266294",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 32924,
            "range": "± 175",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 772115,
            "range": "± 3063",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 12560933,
            "range": "± 279951",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 30138656,
            "range": "± 1664808",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ivanov@pesho.info",
            "name": "Pesho Ivanov",
            "username": "pesho-ivanov"
          },
          "committer": {
            "email": "ivanov@pesho.info",
            "name": "Pesho Ivanov",
            "username": "pesho-ivanov"
          },
          "distinct": true,
          "id": "e78b091d5dcb91899810250aea4e3b1e00488dee",
          "message": "shapes",
          "timestamp": "2022-04-18T17:51:18+02:00",
          "tree_id": "d1f0f07e8d5460ad8feb76a3839bbef7ff00605b",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/e78b091d5dcb91899810250aea4e3b1e00488dee"
        },
        "date": 1650297297237,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 36820,
            "range": "± 176",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 869162,
            "range": "± 8754",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 13096937,
            "range": "± 71871",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 30873091,
            "range": "± 344287",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 33389,
            "range": "± 217",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 796160,
            "range": "± 8747",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 13364422,
            "range": "± 59985",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 26821056,
            "range": "± 505434",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "d7e01a0cdb8eaa20dacdda30461ef77da9586693",
          "message": "snakemake: move unused real-data rules to the bottom",
          "timestamp": "2022-04-18T18:03:33+02:00",
          "tree_id": "62d5833b248ad7f3934794d40abba5eb5834b63d",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/d7e01a0cdb8eaa20dacdda30461ef77da9586693"
        },
        "date": 1650298024491,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 36677,
            "range": "± 170",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 862355,
            "range": "± 3148",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 12546530,
            "range": "± 77780",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 31906946,
            "range": "± 1813059",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 32459,
            "range": "± 283",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 770954,
            "range": "± 2555",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 12250648,
            "range": "± 56715",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 26987579,
            "range": "± 1798499",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ivanov@pesho.info",
            "name": "Pesho Ivanov",
            "username": "pesho-ivanov"
          },
          "committer": {
            "email": "ivanov@pesho.info",
            "name": "Pesho Ivanov",
            "username": "pesho-ivanov"
          },
          "distinct": true,
          "id": "049ae3700c6e885c481a637128dbd07a6547c28e",
          "message": "colors",
          "timestamp": "2022-04-18T18:54:29+02:00",
          "tree_id": "e018c5a44df47ff4ea4e601f62c3fd4d34b8fdc3",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/049ae3700c6e885c481a637128dbd07a6547c28e"
        },
        "date": 1650301101969,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 37391,
            "range": "± 552",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 867515,
            "range": "± 9890",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 13093537,
            "range": "± 298560",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 38534658,
            "range": "± 1902865",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 33270,
            "range": "± 284",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 774664,
            "range": "± 5374",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 13107057,
            "range": "± 443952",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 34284407,
            "range": "± 1873704",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "1be5c6136ddc0deb09d4298c8a680383cda6738c",
          "message": "Cleaner k formula",
          "timestamp": "2022-04-18T19:07:59+02:00",
          "tree_id": "48d2155c2bc7b3360631059acaa97b47eeb1052c",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/1be5c6136ddc0deb09d4298c8a680383cda6738c"
        },
        "date": 1650302183410,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 37001,
            "range": "± 269",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 863749,
            "range": "± 7614",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 12891826,
            "range": "± 355444",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 35318702,
            "range": "± 973459",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 32380,
            "range": "± 137",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 772288,
            "range": "± 2979",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 12642772,
            "range": "± 270827",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 30899408,
            "range": "± 1331396",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "421424e7ec5d1f61263cd89f8bd4132b5764c6e4",
          "message": "Remove 1e6 tables",
          "timestamp": "2022-04-18T20:12:36+02:00",
          "tree_id": "9483d4e70959273caae67e7fef471f5f8aa96b34",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/421424e7ec5d1f61263cd89f8bd4132b5764c6e4"
        },
        "date": 1650305813666,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 36863,
            "range": "± 189",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 863585,
            "range": "± 6495",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 12675507,
            "range": "± 274530",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 33575803,
            "range": "± 2978159",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 32781,
            "range": "± 99",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 772894,
            "range": "± 2929",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 12357264,
            "range": "± 447813",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 27464705,
            "range": "± 1679611",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "cc0ed812b2f24e631b304f61b7cde1f80e3bc685",
          "message": "Update evals to new pa / pa_manual naming scheme",
          "timestamp": "2022-04-18T20:13:19+02:00",
          "tree_id": "3f5101ef9712d03d60b79016f852d0f868efcdfe",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/cc0ed812b2f24e631b304f61b7cde1f80e3bc685"
        },
        "date": 1650305831944,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 36846,
            "range": "± 197",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 865341,
            "range": "± 15363",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 13465814,
            "range": "± 994966",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 35720821,
            "range": "± 4732869",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 32370,
            "range": "± 152",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 771590,
            "range": "± 3095",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 12364059,
            "range": "± 380005",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 31321934,
            "range": "± 5450273",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "9c1bb91fe6ac0fd881020edbcfe9d4813135fc35",
          "message": "Add tools_slow for Dijkstra and no-prune variants",
          "timestamp": "2022-04-18T21:12:03+02:00",
          "tree_id": "1fca77ca77ccae79cbab021e5e783fb40249aada",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/9c1bb91fe6ac0fd881020edbcfe9d4813135fc35"
        },
        "date": 1650309348398,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 37820,
            "range": "± 346",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 871875,
            "range": "± 13288",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 13344037,
            "range": "± 350924",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 34569305,
            "range": "± 1685577",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 33614,
            "range": "± 258",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 796058,
            "range": "± 7231",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 13715459,
            "range": "± 440481",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 30072759,
            "range": "± 1595557",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ivanov@pesho.info",
            "name": "Pesho Ivanov",
            "username": "pesho-ivanov"
          },
          "committer": {
            "email": "ivanov@pesho.info",
            "name": "Pesho Ivanov",
            "username": "pesho-ivanov"
          },
          "distinct": true,
          "id": "7b6f944970acc3c2f7e23011fb8391c739abb064",
          "message": "green fig1",
          "timestamp": "2022-04-18T21:13:00+02:00",
          "tree_id": "fb52a940f8798d830fa02a5fc3b8c7d5eb5b0541",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/7b6f944970acc3c2f7e23011fb8391c739abb064"
        },
        "date": 1650309400752,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 37292,
            "range": "± 616",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 863755,
            "range": "± 10215",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 12794679,
            "range": "± 382287",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 35852641,
            "range": "± 2406806",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 32436,
            "range": "± 156",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 772654,
            "range": "± 3182",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 12558038,
            "range": "± 474822",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 32324717,
            "range": "± 3161969",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "4f05af799b850b913564b764401a492c9803b57a",
          "message": "improve plots",
          "timestamp": "2022-04-18T22:29:03+02:00",
          "tree_id": "4326ef8f6ab6e30928ecbb63ca2c953ff465cdb9",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/4f05af799b850b913564b764401a492c9803b57a"
        },
        "date": 1650313965367,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 36943,
            "range": "± 1259",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 861800,
            "range": "± 6333",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 12620356,
            "range": "± 303202",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 33635412,
            "range": "± 4148564",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 32270,
            "range": "± 80",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 772127,
            "range": "± 2699",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 12327365,
            "range": "± 81681",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 27805615,
            "range": "± 3276385",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "049256cd75f7d0de59a70ab6a128316e8b726ba7",
          "message": "Clean header.py and add s_per_bp column",
          "timestamp": "2022-04-18T22:37:47+02:00",
          "tree_id": "6e89ffbeeeefc335ee97edac714849a01f6e3837",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/049256cd75f7d0de59a70ab6a128316e8b726ba7"
        },
        "date": 1650314497159,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 36954,
            "range": "± 337",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 863811,
            "range": "± 174616",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 12791712,
            "range": "± 330801",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 34030055,
            "range": "± 1698174",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 32443,
            "range": "± 126",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 773090,
            "range": "± 2466",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 12434349,
            "range": "± 286054",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 29440063,
            "range": "± 1787481",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "ffba321c8cc82b3ee8686d1dff04ab22e3c73ce8",
          "message": "Add time per basepair plots",
          "timestamp": "2022-04-18T22:54:23+02:00",
          "tree_id": "fced5285628d54e81ecbce0174ecc16307705ebb",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/ffba321c8cc82b3ee8686d1dff04ab22e3c73ce8"
        },
        "date": 1650315487375,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 37076,
            "range": "± 415",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 863461,
            "range": "± 6440",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 12761671,
            "range": "± 197280",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 34082536,
            "range": "± 2502910",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 32532,
            "range": "± 166",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 774699,
            "range": "± 3828",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 12479455,
            "range": "± 183309",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 29470262,
            "range": "± 982696",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "c1385dbf07765b84dc5d249877bfda5764930525",
          "message": "New 1e7 run",
          "timestamp": "2022-04-19T00:30:26+02:00",
          "tree_id": "c08b4fb97405f264f15d741ec552c89f8b5b8040",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/c1385dbf07765b84dc5d249877bfda5764930525"
        },
        "date": 1650321245751,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 37514,
            "range": "± 312",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 865274,
            "range": "± 10087",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 12778060,
            "range": "± 495073",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 36005553,
            "range": "± 3110952",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 32831,
            "range": "± 153",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 773461,
            "range": "± 3167",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 12544110,
            "range": "± 288488",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 31658772,
            "range": "± 3083271",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "19cbba95bf2e14cdf15fa44e912ec4d8d2b4c069",
          "message": "Do not merge params into tools table",
          "timestamp": "2022-04-19T00:39:54+02:00",
          "tree_id": "5a238d10533cf462422433bdf70a0e9316f84e26",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/19cbba95bf2e14cdf15fa44e912ec4d8d2b4c069"
        },
        "date": 1650321805886,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 37790,
            "range": "± 676",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 863635,
            "range": "± 7951",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 12534406,
            "range": "± 67874",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 31790571,
            "range": "± 1746562",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 32450,
            "range": "± 90",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 772244,
            "range": "± 2197",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 12300054,
            "range": "± 35697",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 27211688,
            "range": "± 1526310",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "07eab815aae62d38b31daeaa394dec4253b35e97",
          "message": "New slightly faster 1e7 tables from 3-parallel run",
          "timestamp": "2022-04-19T08:04:30+02:00",
          "tree_id": "2debb350f34157a6cc296624c24a1ff7e1bb441d",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/07eab815aae62d38b31daeaa394dec4253b35e97"
        },
        "date": 1650348510560,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 38053,
            "range": "± 984",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 861621,
            "range": "± 9518",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 12511937,
            "range": "± 197397",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 31481879,
            "range": "± 3602468",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 33014,
            "range": "± 128",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 766298,
            "range": "± 2409",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 12218701,
            "range": "± 164135",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 27056921,
            "range": "± 2931109",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "80bbf04d4c22b4f769d988dad0fd7083373738f3",
          "message": "Add 1e6 and 1e7 params tables",
          "timestamp": "2022-04-19T08:59:00+02:00",
          "tree_id": "5ed91e01d654757b9e737ac5420c7ce6d4c70769",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/80bbf04d4c22b4f769d988dad0fd7083373738f3"
        },
        "date": 1650351773215,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 37039,
            "range": "± 274",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 859935,
            "range": "± 27210",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 12712852,
            "range": "± 371240",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 35790347,
            "range": "± 4045173",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 32306,
            "range": "± 257",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 765444,
            "range": "± 3004",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 12355277,
            "range": "± 326639",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 31200724,
            "range": "± 3975748",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "ae1f32d42465786610a01745db3ea4a3ff1716c1",
          "message": "Use shell: instead of run: rules to prevent python memory overhead",
          "timestamp": "2022-04-19T11:29:44+02:00",
          "tree_id": "5dad7d8c49ec91ad22f263c6599b1a674ae05d6e",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/ae1f32d42465786610a01745db3ea4a3ff1716c1"
        },
        "date": 1650360818820,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 36860,
            "range": "± 164",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 859335,
            "range": "± 5223",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 12591188,
            "range": "± 175563",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 33673835,
            "range": "± 1856542",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 32322,
            "range": "± 280",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 764885,
            "range": "± 2253",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 12292018,
            "range": "± 275516",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 28723843,
            "range": "± 1962556",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "b410ffe67fc0ac45a5ccbb150f04ff017647b13e",
          "message": "Update max_uss column name",
          "timestamp": "2022-04-19T11:39:43+02:00",
          "tree_id": "f9002fb52a65b403f5def882be96ab4b855e1bb6",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/b410ffe67fc0ac45a5ccbb150f04ff017647b13e"
        },
        "date": 1650361408654,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 36943,
            "range": "± 323",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 857969,
            "range": "± 4333",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 12596722,
            "range": "± 241930",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 34503463,
            "range": "± 4096776",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 32374,
            "range": "± 144",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 763586,
            "range": "± 2463",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 12231192,
            "range": "± 215133",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 28131286,
            "range": "± 3473449",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "ce869555393e797e9d847fe83aa2d6dc76e2a613",
          "message": "Add --no-greedy-matching flag that replaces the config setting.",
          "timestamp": "2022-04-19T20:24:41+02:00",
          "tree_id": "ee28b9b70764d18415042eb6d5b24c35fa781d59",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/ce869555393e797e9d847fe83aa2d6dc76e2a613"
        },
        "date": 1650392945111,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 57681,
            "range": "± 364",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1223018,
            "range": "± 9685",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 16596258,
            "range": "± 1404122",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 52907159,
            "range": "± 5509813",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 47566,
            "range": "± 243",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 1054026,
            "range": "± 3648",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 16899222,
            "range": "± 457341",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 47783776,
            "range": "± 3982908",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "e330225e3d603cda64e596cbd5ef1a7402368380",
          "message": "New 20% visualizations, and dijkstra-nogreedy",
          "timestamp": "2022-04-19T23:57:18+02:00",
          "tree_id": "61c11373c8d76758db9541dcd29d7148a335e045",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/e330225e3d603cda64e596cbd5ef1a7402368380"
        },
        "date": 1650405670531,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 37266,
            "range": "± 197",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 864067,
            "range": "± 4941",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 12585158,
            "range": "± 222035",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 34110266,
            "range": "± 3019944",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 32511,
            "range": "± 176",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 767015,
            "range": "± 2485",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 12399154,
            "range": "± 538998",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 28773600,
            "range": "± 3029035",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ivanov@pesho.info",
            "name": "Pesho Ivanov",
            "username": "pesho-ivanov"
          },
          "committer": {
            "email": "ivanov@pesho.info",
            "name": "Pesho Ivanov",
            "username": "pesho-ivanov"
          },
          "distinct": true,
          "id": "c84e67f4ed347bbe04e1b91b1e4bfd35b50ff9d7",
          "message": "colormaps",
          "timestamp": "2022-04-20T00:07:21+02:00",
          "tree_id": "99381f7c79f9508043e82cde0b0d069de05c5de5",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/c84e67f4ed347bbe04e1b91b1e4bfd35b50ff9d7"
        },
        "date": 1650406257988,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 37596,
            "range": "± 278",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 862610,
            "range": "± 6826",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 12639582,
            "range": "± 285790",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 32009466,
            "range": "± 2960380",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 32499,
            "range": "± 132",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 768449,
            "range": "± 3629",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 12229047,
            "range": "± 143052",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 26394378,
            "range": "± 1450135",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "9ffa3b2feb911e7117b7190997fc08110a34e94f",
          "message": "todos",
          "timestamp": "2022-04-20T00:14:23+02:00",
          "tree_id": "a75f9d446e09cea22f1650a10c3041d57cbccd28",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/9ffa3b2feb911e7117b7190997fc08110a34e94f"
        },
        "date": 1650406672291,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 37287,
            "range": "± 169",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 864211,
            "range": "± 5977",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 12486819,
            "range": "± 88822",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 31016423,
            "range": "± 1925630",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 32435,
            "range": "± 342",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 765990,
            "range": "± 2366",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 12183132,
            "range": "± 97688",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 26720046,
            "range": "± 1194570",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ivanov@pesho.info",
            "name": "Pesho Ivanov",
            "username": "pesho-ivanov"
          },
          "committer": {
            "email": "ivanov@pesho.info",
            "name": "Pesho Ivanov",
            "username": "pesho-ivanov"
          },
          "distinct": true,
          "id": "82be4ed9b22b13639744f05971045656f233d57a",
          "message": "new colorscheme for fig.1",
          "timestamp": "2022-04-20T00:54:44+02:00",
          "tree_id": "085dff51347f89bae33894d5bb287019a3a066af",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/82be4ed9b22b13639744f05971045656f233d57a"
        },
        "date": 1650409172309,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 44654,
            "range": "± 1894",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 996305,
            "range": "± 78694",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 14105578,
            "range": "± 1717673",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 33741187,
            "range": "± 5383844",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 37847,
            "range": "± 2797",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 880992,
            "range": "± 110812",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 13268352,
            "range": "± 2021029",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 28630132,
            "range": "± 2713137",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "6bc18a08983820e4e8135b3a4749ca72a59d3c56",
          "message": "new evals",
          "timestamp": "2022-04-20T08:33:54+02:00",
          "tree_id": "e205486ccb053607a1f2bbd0ff237f1636b4c2b9",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/6bc18a08983820e4e8135b3a4749ca72a59d3c56"
        },
        "date": 1650436654900,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 37088,
            "range": "± 144",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 871729,
            "range": "± 7701",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 13125205,
            "range": "± 69670",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 31885131,
            "range": "± 1037019",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 33091,
            "range": "± 201",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 790336,
            "range": "± 8017",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 13329465,
            "range": "± 69958",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 26587021,
            "range": "± 440531",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ivanov@pesho.info",
            "name": "Pesho Ivanov",
            "username": "pesho-ivanov"
          },
          "committer": {
            "email": "ivanov@pesho.info",
            "name": "Pesho Ivanov",
            "username": "pesho-ivanov"
          },
          "distinct": true,
          "id": "a4dc96c3dc81397958e9b23f4f932e179d332e09",
          "message": "old",
          "timestamp": "2022-04-22T03:54:40+02:00",
          "tree_id": "571d120bf4ddf8cb437013692075b7442ef83e96",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/a4dc96c3dc81397958e9b23f4f932e179d332e09"
        },
        "date": 1650592701558,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 37024,
            "range": "± 261",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 875151,
            "range": "± 14495",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 13162590,
            "range": "± 58561",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 31558585,
            "range": "± 1092967",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 33210,
            "range": "± 268",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 788938,
            "range": "± 8101",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 13271530,
            "range": "± 90531",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 26934547,
            "range": "± 650145",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "e3d279a1895199951e1a25607e5c65e7d45f63de",
          "message": "Add nanosim to .seq converter",
          "timestamp": "2022-04-22T03:57:32+02:00",
          "tree_id": "f869a827c28e3e0a4c437a590e1370496b51ebb8",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/e3d279a1895199951e1a25607e5c65e7d45f63de"
        },
        "date": 1650592867177,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 37577,
            "range": "± 274",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 872786,
            "range": "± 7822",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 13170945,
            "range": "± 59244",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 31240181,
            "range": "± 325359",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 35707,
            "range": "± 341",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 787272,
            "range": "± 7211",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 13280785,
            "range": "± 57570",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 26811345,
            "range": "± 443406",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "f8e8fa6b230d730e6c517117a0ddee98f3bd55e6",
          "message": "HOTFIX BUG: BruteForce Contours would grow large with duplicated points????",
          "timestamp": "2022-04-22T10:03:42+02:00",
          "tree_id": "767037cde1702a1f61a7c73023dd563039cb83cd",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/f8e8fa6b230d730e6c517117a0ddee98f3bd55e6"
        },
        "date": 1650614862775,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 36235,
            "range": "± 136",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 853898,
            "range": "± 3101",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 12503411,
            "range": "± 35354",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 30759633,
            "range": "± 1616828",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 33155,
            "range": "± 93",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 762767,
            "range": "± 2184",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 12139583,
            "range": "± 21239",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 26157137,
            "range": "± 541770",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "ba895d2dc7620a97d040ca3013c07c6b540c3a0b",
          "message": "FIX HINT_CONTOURS BUG: It was pushing already present points on contour layers",
          "timestamp": "2022-04-23T19:40:43+02:00",
          "tree_id": "972aafca2bab5083023870741fbf07d5f55a87c7",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/ba895d2dc7620a97d040ca3013c07c6b540c3a0b"
        },
        "date": 1650735885550,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 36407,
            "range": "± 301",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 856041,
            "range": "± 14230",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 12740363,
            "range": "± 258903",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 34739561,
            "range": "± 3997333",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 31679,
            "range": "± 173",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 759470,
            "range": "± 3505",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 12304554,
            "range": "± 433540",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 28972086,
            "range": "± 4250623",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "6ef2ce0a31359f153891db44eceb32bec2ae795c",
          "message": "add table and todos",
          "timestamp": "2022-04-29T11:23:17+02:00",
          "tree_id": "1431ed8241272b15861ccb9539d1b50d88cee238",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/6ef2ce0a31359f153891db44eceb32bec2ae795c"
        },
        "date": 1651224433476,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 36541,
            "range": "± 190",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 857727,
            "range": "± 2724",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 12790609,
            "range": "± 435075",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 33977183,
            "range": "± 2570836",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 32425,
            "range": "± 82",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 758959,
            "range": "± 3110",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 12194553,
            "range": "± 252435",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 28719561,
            "range": "± 2804873",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "e14af0d7c41e461c25f0cd64a7995eca402a043c",
          "message": "clean up asserts",
          "timestamp": "2022-04-29T21:50:14+02:00",
          "tree_id": "177921d456ead33906a4b1f9ec9a639af0577144",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/e14af0d7c41e461c25f0cd64a7995eca402a043c"
        },
        "date": 1651262326630,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 37302,
            "range": "± 419",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 886025,
            "range": "± 10842",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 13280443,
            "range": "± 199342",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 33416375,
            "range": "± 1205258",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 32689,
            "range": "± 277",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 801193,
            "range": "± 13079",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 12988311,
            "range": "± 220469",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 29656379,
            "range": "± 2045674",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ivanov@pesho.info",
            "name": "Pesho Ivanov",
            "username": "pesho-ivanov"
          },
          "committer": {
            "email": "ivanov@pesho.info",
            "name": "Pesho Ivanov",
            "username": "pesho-ivanov"
          },
          "distinct": true,
          "id": "21644e6a5a928c59ddb4f1f838daace04030aedd",
          "message": "upset plots",
          "timestamp": "2022-04-30T13:33:03+02:00",
          "tree_id": "f2ce15649418e494a35a56a5a4a7331e72ecf869",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/21644e6a5a928c59ddb4f1f838daace04030aedd"
        },
        "date": 1651318604058,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 37847,
            "range": "± 502",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 861361,
            "range": "± 6469",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 12671400,
            "range": "± 157290",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 31358135,
            "range": "± 1202175",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 33024,
            "range": "± 103",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 773400,
            "range": "± 7371",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 11913917,
            "range": "± 53369",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 27718696,
            "range": "± 1259245",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ivanov@pesho.info",
            "name": "Pesho Ivanov",
            "username": "pesho-ivanov"
          },
          "committer": {
            "email": "ivanov@pesho.info",
            "name": "Pesho Ivanov",
            "username": "pesho-ivanov"
          },
          "distinct": true,
          "id": "39f7c54f2aa9bec496fa24dd90869667ea2c3a10",
          "message": "nit",
          "timestamp": "2022-04-30T13:51:44+02:00",
          "tree_id": "3cf9b47f54820c14077732530e9d320f89e880f5",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/39f7c54f2aa9bec496fa24dd90869667ea2c3a10"
        },
        "date": 1651319737044,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 37951,
            "range": "± 1990",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 862029,
            "range": "± 10217",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 13292803,
            "range": "± 377826",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 39050678,
            "range": "± 2447337",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 32920,
            "range": "± 112",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 773508,
            "range": "± 4895",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 12638645,
            "range": "± 582084",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 35070205,
            "range": "± 2308403",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "bfaef84ec369ca37d165ca01109cc93f81e8509c",
          "message": "ONT tools/param runs and snakemake update",
          "timestamp": "2022-05-03T12:16:22+02:00",
          "tree_id": "740b46411747bc31c23150aca79732d34d2d0c6a",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/bfaef84ec369ca37d165ca01109cc93f81e8509c"
        },
        "date": 1651573203520,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 33439,
            "range": "± 552",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 834957,
            "range": "± 11300",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 15975720,
            "range": "± 285816",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 37437098,
            "range": "± 2131667",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 29844,
            "range": "± 463",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 745834,
            "range": "± 5374",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 14503990,
            "range": "± 110873",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 32447620,
            "range": "± 4146467",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "857f3f8888a4a198cd2eae2532c5e3e598825a66",
          "message": "Auto choose params for SH/CSH based on new analysis",
          "timestamp": "2022-05-04T03:36:01+02:00",
          "tree_id": "c8339945e4ec4337e71b7fdb12678e38394923e5",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/857f3f8888a4a198cd2eae2532c5e3e598825a66"
        },
        "date": 1651628420421,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 39008,
            "range": "± 187",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 980027,
            "range": "± 10236",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 18691452,
            "range": "± 37376",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 40754827,
            "range": "± 454566",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 34766,
            "range": "± 437",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 900840,
            "range": "± 15327",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 18025808,
            "range": "± 585424",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 36753380,
            "range": "± 411494",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "dc5ad49cac768692164279f2fd80dac579a7f04a",
          "message": "Add tools table with timeout 100s",
          "timestamp": "2022-05-04T09:25:19+02:00",
          "tree_id": "290d83f0bc1ff218cee2c2a9ae40a9a370828118",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/dc5ad49cac768692164279f2fd80dac579a7f04a"
        },
        "date": 1651649364882,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 39566,
            "range": "± 639",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 980393,
            "range": "± 19994",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 18760665,
            "range": "± 289084",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 43087257,
            "range": "± 1111504",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 34966,
            "range": "± 544",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 902550,
            "range": "± 10146",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 18087512,
            "range": "± 753926",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 38734585,
            "range": "± 1836347",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "0a472f17cf9c28daeb9837e47f121027b2244b46",
          "message": "Rename algorithms; remove -nogreedy variants",
          "timestamp": "2022-05-04T10:27:05+02:00",
          "tree_id": "6ed3359c7a4d309315b4b645ad10eaf7e1992076",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/0a472f17cf9c28daeb9837e47f121027b2244b46"
        },
        "date": 1651653069364,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 38440,
            "range": "± 250",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 949068,
            "range": "± 92085",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 18208775,
            "range": "± 339052",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 44093629,
            "range": "± 2049632",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 33983,
            "range": "± 211",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 856027,
            "range": "± 3447",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 17163625,
            "range": "± 343065",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 40203495,
            "range": "± 2971031",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "91ffbf528c408d8b8bc079c5c1053807321da377",
          "message": "Add band table",
          "timestamp": "2022-05-04T11:41:26+02:00",
          "tree_id": "d526369edb999231a2b0ac6adbe0aed3eb26fbf7",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/91ffbf528c408d8b8bc079c5c1053807321da377"
        },
        "date": 1651657522943,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 40207,
            "range": "± 215",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 978851,
            "range": "± 10262",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 18675137,
            "range": "± 137063",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 42235928,
            "range": "± 1913667",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 35261,
            "range": "± 251",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 901402,
            "range": "± 34344",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 18119656,
            "range": "± 471240",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 38600539,
            "range": "± 1888015",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "756f145f64721a4dbd8094422ec83735b4dccd79",
          "message": "Update fig 1 visualization and naming",
          "timestamp": "2022-05-04T17:37:24+02:00",
          "tree_id": "27c9bc8970a7781b8be057e9d824a8fa31e880d5",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/756f145f64721a4dbd8094422ec83735b4dccd79"
        },
        "date": 1651678979263,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 38449,
            "range": "± 12804",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 947764,
            "range": "± 12216",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 19126846,
            "range": "± 545667",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 48078400,
            "range": "± 949577",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 33909,
            "range": "± 188",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 851351,
            "range": "± 4043",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 17914336,
            "range": "± 31351055",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 42801460,
            "range": "± 637178",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "2273c5721efd7e72355f2394f99557116f072d53",
          "message": "Fix tests by copying pruning logic from CSH to BruteforceCSH",
          "timestamp": "2022-05-04T17:51:33+02:00",
          "tree_id": "773f714f47649bb0710b6d3234e60d7ef3ec333f",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/2273c5721efd7e72355f2394f99557116f072d53"
        },
        "date": 1651679735347,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 38691,
            "range": "± 781",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 945326,
            "range": "± 4738",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 17842255,
            "range": "± 86174",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 40300993,
            "range": "± 1711865",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 34001,
            "range": "± 126",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 851566,
            "range": "± 3173",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 16794248,
            "range": "± 191848",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 36282291,
            "range": "± 2370898",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "4e4be1c79334f2480609c3e64f1172801bb725fc",
          "message": "Fix potential overflow/infinite loop in parent computation",
          "timestamp": "2022-05-05T01:21:21+02:00",
          "tree_id": "f962f3252df141a3558f65c05287faf06d7a36eb",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/4e4be1c79334f2480609c3e64f1172801bb725fc"
        },
        "date": 1651706770984,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 38447,
            "range": "± 200",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 980011,
            "range": "± 9463",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 19292857,
            "range": "± 91650",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 45300965,
            "range": "± 493856",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 34702,
            "range": "± 311",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 901799,
            "range": "± 8793",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 18671216,
            "range": "± 89899",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 41605677,
            "range": "± 642824",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "4d2dce485a93c15efa17abefabf73335dbb15356",
          "message": "new tools_N1e7 run, 1000s timeout. Final data?",
          "timestamp": "2022-05-05T09:52:54+02:00",
          "tree_id": "79a3b4d5ffa33b029d4efc1725d8d675a153267d",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/4d2dce485a93c15efa17abefabf73335dbb15356"
        },
        "date": 1651737454610,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 39007,
            "range": "± 1827",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 978112,
            "range": "± 9054",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 19001984,
            "range": "± 198537",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 42585782,
            "range": "± 1563663",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 34530,
            "range": "± 343",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 891470,
            "range": "± 15642",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 17790233,
            "range": "± 276577",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 36967755,
            "range": "± 1902790",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ivanov@pesho.info",
            "name": "Pesho Ivanov",
            "username": "pesho-ivanov"
          },
          "committer": {
            "email": "ivanov@pesho.info",
            "name": "Pesho Ivanov",
            "username": "pesho-ivanov"
          },
          "distinct": true,
          "id": "fbf3ce6426ccb4e77b8992541e54ae69a5537c36",
          "message": "cleanup",
          "timestamp": "2022-05-06T07:43:02+02:00",
          "tree_id": "c2ce8eb7baa5b806515265d9970bef43a8b870db",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/fbf3ce6426ccb4e77b8992541e54ae69a5537c36"
        },
        "date": 1651816069819,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 44424,
            "range": "± 12006",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1029723,
            "range": "± 167603",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 19685939,
            "range": "± 2530846",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 47168421,
            "range": "± 5260538",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 37996,
            "range": "± 10286",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 853451,
            "range": "± 160958",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 16826676,
            "range": "± 2206674",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 37774144,
            "range": "± 8658062",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "c96fc1b6a4715bc9dce61b5d8eef78437c1e4d55",
          "message": "Test new CI",
          "timestamp": "2022-05-06T14:34:13+02:00",
          "tree_id": "a3bf254410abf3ffc3888dd43a14fc2b82424825",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/c96fc1b6a4715bc9dce61b5d8eef78437c1e4d55"
        },
        "date": 1651840692567,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 42162,
            "range": "± 159",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1027170,
            "range": "± 8948",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 19106703,
            "range": "± 66947",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 44010125,
            "range": "± 1293346",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 37731,
            "range": "± 295",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 925638,
            "range": "± 12802",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 17711759,
            "range": "± 253957",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 38511406,
            "range": "± 728017",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "b1c00287e7b244cbc5b88cd21da908dc8316b5e0",
          "message": "Disable default features on GitHub CI",
          "timestamp": "2022-05-06T14:36:48+02:00",
          "tree_id": "32cacc31a82a9c76c315e97080324aa41d435f68",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/b1c00287e7b244cbc5b88cd21da908dc8316b5e0"
        },
        "date": 1651840850167,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 42626,
            "range": "± 269",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1029755,
            "range": "± 14263",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 19137160,
            "range": "± 55338",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 43684879,
            "range": "± 1016457",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 37733,
            "range": "± 271",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 931012,
            "range": "± 11096",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 17758460,
            "range": "± 420779",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 38405646,
            "range": "± 703150",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "5b7737387a51af1d89c1a3003ce2f97e53d41371",
          "message": "Report traceback time",
          "timestamp": "2022-05-06T16:11:02+02:00",
          "tree_id": "7ecece10f59a7a50648a180c45f08efc40292e03",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/5b7737387a51af1d89c1a3003ce2f97e53d41371"
        },
        "date": 1651847723971,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 41953,
            "range": "± 225",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1032793,
            "range": "± 14036",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 19089445,
            "range": "± 132093",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 44697931,
            "range": "± 1654375",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 37713,
            "range": "± 143",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 925517,
            "range": "± 18992",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 17502084,
            "range": "± 176157",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 38443544,
            "range": "± 1367341",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "8c8b702053ae482ed31769f2506e801b7f1d91f0",
          "message": "Clean draw function",
          "timestamp": "2022-05-06T19:15:10+02:00",
          "tree_id": "4e4ef08b34934cdcba6e81046f5811e4a2eb65ef",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/8c8b702053ae482ed31769f2506e801b7f1d91f0"
        },
        "date": 1651858005510,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 42620,
            "range": "± 599",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1030168,
            "range": "± 13321",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 19050168,
            "range": "± 55651",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 43362571,
            "range": "± 534074",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 37403,
            "range": "± 214",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 927783,
            "range": "± 14302",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 17506463,
            "range": "± 78752",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 37934620,
            "range": "± 496796",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": false,
          "id": "985b87e87aea438a1fe47821efc80f8770ac5ff7",
          "message": "Remove termion dep to support windows",
          "timestamp": "2022-05-06T19:23:18+02:00",
          "tree_id": "e1452d34a9fa141c503a68a9784656b7e8727af4",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/985b87e87aea438a1fe47821efc80f8770ac5ff7"
        },
        "date": 1651859627784,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 42830,
            "range": "± 418",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1023712,
            "range": "± 13611",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 18921713,
            "range": "± 136829",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 44929272,
            "range": "± 2030525",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 38316,
            "range": "± 257",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 937184,
            "range": "± 12350",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 17580042,
            "range": "± 206426",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 39675933,
            "range": "± 1502365",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "2841c7a89ab8e648ec4ab80d13bd6a1d031dd00b",
          "message": "Smaller sample",
          "timestamp": "2022-05-07T01:01:40+02:00",
          "tree_id": "6cb6a65752afc38396648f1536a425984a9c5912",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/2841c7a89ab8e648ec4ab80d13bd6a1d031dd00b"
        },
        "date": 1651878387520,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 48572,
            "range": "± 9562",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 979804,
            "range": "± 195701",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 18336826,
            "range": "± 3578292",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 48674798,
            "range": "± 10725677",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 41398,
            "range": "± 8019",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 902871,
            "range": "± 170621",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 16901146,
            "range": "± 2595234",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 43011992,
            "range": "± 9471993",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "fd556db51854a6b4d72f4389764efadd3d383223",
          "message": "Draw tree",
          "timestamp": "2022-05-07T12:58:48+02:00",
          "tree_id": "dbd893aec121b7526384ab03a75b67e31befd7bc",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/fd556db51854a6b4d72f4389764efadd3d383223"
        },
        "date": 1651921360916,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 43424,
            "range": "± 206",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1040661,
            "range": "± 12788",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 19243259,
            "range": "± 125577",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 45337233,
            "range": "± 1483517",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 37784,
            "range": "± 371",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 931308,
            "range": "± 11899",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 17535894,
            "range": "± 154774",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 39886965,
            "range": "± 2202169",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "8f4de82e52798f726ee02b4ac0e73ee5ac289389",
          "message": "Add scaling n/k/e to snakefile and tables",
          "timestamp": "2022-05-12T13:13:00+02:00",
          "tree_id": "711b86e1b95829fa890db59a8ccc1db7b0fce5f7",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/8f4de82e52798f726ee02b4ac0e73ee5ac289389"
        },
        "date": 1652354252253,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 42248,
            "range": "± 339",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1036383,
            "range": "± 10665",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 19421832,
            "range": "± 241648",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 46241212,
            "range": "± 2032975",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 37898,
            "range": "± 268",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 932322,
            "range": "± 5973",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 17848637,
            "range": "± 209231",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 40291949,
            "range": "± 980752",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "456220ce3af66daf0daf1e5c7f3c74a7d018591d",
          "message": "FINAL 1h TIMEOUT TOOLS RUN",
          "timestamp": "2022-05-15T06:51:32-07:00",
          "tree_id": "67ac826b3eaa7c578996f73ed1047428b5e1b55f",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/456220ce3af66daf0daf1e5c7f3c74a7d018591d"
        },
        "date": 1652622954024,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 41743,
            "range": "± 371",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1026750,
            "range": "± 12612",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 18847993,
            "range": "± 58499",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 43213137,
            "range": "± 919653",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 38115,
            "range": "± 288",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 937815,
            "range": "± 11115",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 17523672,
            "range": "± 426185",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 38939485,
            "range": "± 1751693",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "89f22bc5aa814b9be6dcf489d4bd6569a5de7b3b",
          "message": "evals.py: clean filenames",
          "timestamp": "2022-05-15T11:00:17-07:00",
          "tree_id": "2a0604d4417f51577dde6ea8fef595d1186dbc60",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/89f22bc5aa814b9be6dcf489d4bd6569a5de7b3b"
        },
        "date": 1652637869157,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 41422,
            "range": "± 289",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1022675,
            "range": "± 11738",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 18860400,
            "range": "± 59007",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 43231965,
            "range": "± 566691",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 37917,
            "range": "± 315",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 932116,
            "range": "± 17679",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 17541433,
            "range": "± 384024",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 38149854,
            "range": "± 514766",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "178b60d10b2a2d9f13aec4df67e14d9423d38fc9",
          "message": "NEW TOOLS_N1e7 RUN WITH DIJKSTRA,NOPRUNE",
          "timestamp": "2022-05-16T07:31:55-07:00",
          "tree_id": "51d28874926fdfd8af894f59397099b4526591ec",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/178b60d10b2a2d9f13aec4df67e14d9423d38fc9"
        },
        "date": 1652712229103,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 43176,
            "range": "± 323",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1059442,
            "range": "± 8819",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 19296572,
            "range": "± 394669",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 45142386,
            "range": "± 1423341",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 37957,
            "range": "± 401",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 945981,
            "range": "± 10306",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 17786278,
            "range": "± 739910",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 38950211,
            "range": "± 1054909",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "91976b7749f0fea923d3f56b3c309d08d8418cec",
          "message": "FIX SMALL TIMEOUT RUNS",
          "timestamp": "2022-05-16T11:37:52-07:00",
          "tree_id": "204c0f4d82eca165abc2c3a1bdd887c700ca0825",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/91976b7749f0fea923d3f56b3c309d08d8418cec"
        },
        "date": 1652726539170,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 43214,
            "range": "± 600",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1053454,
            "range": "± 9408",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 19358652,
            "range": "± 156036",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 46050109,
            "range": "± 1056788",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 38177,
            "range": "± 270",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 940606,
            "range": "± 8391",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 17899054,
            "range": "± 671441",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 40154529,
            "range": "± 1635708",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "320a2ac9b1a3fc58e13597965ebdbe5668a5589b",
          "message": "DISABLE UNNEEDED CONSISTENCY CHECKS",
          "timestamp": "2022-05-18T01:18:52-07:00",
          "tree_id": "d57cb69ffdf2b20c4187271c879056d796631428",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/320a2ac9b1a3fc58e13597965ebdbe5668a5589b"
        },
        "date": 1652862164967,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 36461,
            "range": "± 1474",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 784871,
            "range": "± 49744",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 11435836,
            "range": "± 355563",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 29868984,
            "range": "± 2755092",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 31744,
            "range": "± 824",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 706710,
            "range": "± 13120",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 10538873,
            "range": "± 332731",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 26213158,
            "range": "± 2084593",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "64464b716d3d9dbdd60ce296aa50a04507157f51",
          "message": "update evals",
          "timestamp": "2022-05-18T09:15:28-07:00",
          "tree_id": "af5f6ccb38b615e2aa0a39d1629f2faa5eb7f535",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/64464b716d3d9dbdd60ce296aa50a04507157f51"
        },
        "date": 1652890817787,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 49156,
            "range": "± 790",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1068305,
            "range": "± 6457",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 15623334,
            "range": "± 297090",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 42142753,
            "range": "± 1456475",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 42831,
            "range": "± 335",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 960941,
            "range": "± 3545",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 14340963,
            "range": "± 191771",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 37704933,
            "range": "± 1622988",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "7699bbe74d0d8fae5124242122a89a75b8605add",
          "message": "ignore backup of runs",
          "timestamp": "2022-05-18T09:16:58-07:00",
          "tree_id": "ff146864dcda842c8c1013c3fdc6d31d9b723371",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/7699bbe74d0d8fae5124242122a89a75b8605add"
        },
        "date": 1652890893652,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 48551,
            "range": "± 2974",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1066302,
            "range": "± 7919",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 15192422,
            "range": "± 717401",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 39468855,
            "range": "± 6332216",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 42615,
            "range": "± 558",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 948230,
            "range": "± 106422",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 14224240,
            "range": "± 755510",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 34427209,
            "range": "± 5379820",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "0dc7e4ce535f6ac4afa819f81fe6341a01c2509f",
          "message": "Update figures",
          "timestamp": "2022-05-19T20:54:18-07:00",
          "tree_id": "bb905257447a180e5fc49c4d32a675e0a640b9a1",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/0dc7e4ce535f6ac4afa819f81fe6341a01c2509f"
        },
        "date": 1653019159644,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 45412,
            "range": "± 8755",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 983083,
            "range": "± 263281",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 13964819,
            "range": "± 2425566",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 35792175,
            "range": "± 7787181",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 41389,
            "range": "± 9159",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 869367,
            "range": "± 204638",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 13278774,
            "range": "± 1807598",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 34681804,
            "range": "± 10248144",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ivanov@pesho.info",
            "name": "Pesho Ivanov",
            "username": "pesho-ivanov"
          },
          "committer": {
            "email": "ivanov@pesho.info",
            "name": "Pesho Ivanov",
            "username": "pesho-ivanov"
          },
          "distinct": true,
          "id": "6b12368a705ad6b077b83a1d26344db1ecc9a410",
          "message": "e2marker+colors",
          "timestamp": "2022-05-20T06:38:00+02:00",
          "tree_id": "e1af5c1648e35375686753f9341f8a62d11e7ca1",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/6b12368a705ad6b077b83a1d26344db1ecc9a410"
        },
        "date": 1653021971245,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 40370,
            "range": "± 304",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 886260,
            "range": "± 5303",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 12934124,
            "range": "± 283067",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 35379102,
            "range": "± 4585227",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 35803,
            "range": "± 145",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 800390,
            "range": "± 12757",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 12021762,
            "range": "± 141785",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 29544974,
            "range": "± 2070760",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "6f1e58775efe3fc047848a9f5dcaf78056b44fc9",
          "message": "Fix r-dependent marker",
          "timestamp": "2022-05-19T21:57:49-07:00",
          "tree_id": "f4f0ed008d797e6824676c286830fb2bdd2be83d",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/6f1e58775efe3fc047848a9f5dcaf78056b44fc9"
        },
        "date": 1653022967448,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 49342,
            "range": "± 9952",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 923593,
            "range": "± 242387",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 14257327,
            "range": "± 3903244",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 34306357,
            "range": "± 5335953",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 38588,
            "range": "± 7881",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 846900,
            "range": "± 171035",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 12067234,
            "range": "± 2957356",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 30016054,
            "range": "± 4423121",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ivanov@pesho.info",
            "name": "Pesho Ivanov",
            "username": "pesho-ivanov"
          },
          "committer": {
            "email": "ivanov@pesho.info",
            "name": "Pesho Ivanov",
            "username": "pesho-ivanov"
          },
          "distinct": true,
          "id": "3aa915bfb78fec7fddd64e031e2a7824e912e591",
          "message": "colors",
          "timestamp": "2022-05-20T07:18:55+02:00",
          "tree_id": "30faf39b6b6e48ac0a87e1a6b2f843ff098f656b",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/3aa915bfb78fec7fddd64e031e2a7824e912e591"
        },
        "date": 1653024224890,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 49779,
            "range": "± 5368",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1073235,
            "range": "± 274535",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 16068630,
            "range": "± 369485",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 45998914,
            "range": "± 2506875",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 42706,
            "range": "± 247",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 959368,
            "range": "± 5266",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 14954951,
            "range": "± 575893",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 41354162,
            "range": "± 1236121",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "ae22bf3c86ce93bdd2ab1c1756782b73021d6892",
          "message": "swap sh and csh colors",
          "timestamp": "2022-05-19T22:50:04-07:00",
          "tree_id": "9483a7d2bef72e1acd4294a5b99c7a960c4970d8",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/ae22bf3c86ce93bdd2ab1c1756782b73021d6892"
        },
        "date": 1653026067539,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 40231,
            "range": "± 245",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 883939,
            "range": "± 4325",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 12908896,
            "range": "± 91812",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 35058039,
            "range": "± 2483421",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 35671,
            "range": "± 224",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 798678,
            "range": "± 1887",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 12009666,
            "range": "± 168676",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 30750139,
            "range": "± 1943001",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "023fa6763d32a2733c5e3553a9002798f25b5762",
          "message": "Add sh-noprune to pruning figure",
          "timestamp": "2022-05-20T01:20:55-07:00",
          "tree_id": "bfd26a0d84c029fe799d4356e602e76e3075f142",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/023fa6763d32a2733c5e3553a9002798f25b5762"
        },
        "date": 1653035190893,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 48583,
            "range": "± 661",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1060278,
            "range": "± 7298",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 15743378,
            "range": "± 231879",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 45143873,
            "range": "± 2527907",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 42811,
            "range": "± 268",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 958292,
            "range": "± 4386",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 14493128,
            "range": "± 228510",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 39968985,
            "range": "± 2065329",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "67823ab1a9b1e94828c1afa8a9fea3be90bfd638",
          "message": "Fix SH for exact matches",
          "timestamp": "2022-05-20T01:58:08-07:00",
          "tree_id": "4b4ff10e387b2ad31fe9e220290fc59b7efee1c1",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/67823ab1a9b1e94828c1afa8a9fea3be90bfd638"
        },
        "date": 1653037347305,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 41239,
            "range": "± 660",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 885570,
            "range": "± 6629",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 13036109,
            "range": "± 198465",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 36854621,
            "range": "± 4818955",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 35528,
            "range": "± 124",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 799049,
            "range": "± 3207",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 12137448,
            "range": "± 272002",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 31716022,
            "range": "± 2296574",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "05936c0a4d5ff40658cbcb653b1d8ea8fcbde736",
          "message": "New run with more efficient unordered exact match finding",
          "timestamp": "2022-05-20T02:31:58-07:00",
          "tree_id": "9570c1c15bc484c4cccf0e47493fb1d355faeef4",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/05936c0a4d5ff40658cbcb653b1d8ea8fcbde736"
        },
        "date": 1653039405288,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 40335,
            "range": "± 312",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 886146,
            "range": "± 8957",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 13446930,
            "range": "± 774796",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 37061670,
            "range": "± 6406523",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 35452,
            "range": "± 270",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 799055,
            "range": "± 2825",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 12137331,
            "range": "± 228032",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 29899830,
            "range": "± 2872866",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "acb9e0645cc49d9d456190142551a76380e2b7ca",
          "message": "fix bad r values",
          "timestamp": "2022-05-20T02:48:44-07:00",
          "tree_id": "be5c5f82f1afde075efadbfaa8881b047bc0732f",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/acb9e0645cc49d9d456190142551a76380e2b7ca"
        },
        "date": 1653040425601,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 51070,
            "range": "± 5724",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1125005,
            "range": "± 224823",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 16465016,
            "range": "± 1365403",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 44377513,
            "range": "± 2163021",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 44140,
            "range": "± 113",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 999134,
            "range": "± 3376",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 15068626,
            "range": "± 220440",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 39432921,
            "range": "± 2357814",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "fd629f4b521df9b9278dff4556b1ae7a23d1de34",
          "message": "Add -x/--cnt option for random sequence generation",
          "timestamp": "2022-06-02T20:05:59-07:00",
          "tree_id": "a3c72bc78b57a2fdb5019f80a13cca5e1b158473",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/fd629f4b521df9b9278dff4556b1ae7a23d1de34"
        },
        "date": 1654225833303,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 37495,
            "range": "± 9045",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 816108,
            "range": "± 158928",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 11934100,
            "range": "± 1055373",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 30445667,
            "range": "± 3131428",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 33488,
            "range": "± 8283",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 732525,
            "range": "± 77995",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 11065331,
            "range": "± 1087980",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 27156273,
            "range": "± 3618553",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "8ecbb2086c507476ffa2a60f51c50add4bfd43c9",
          "message": "Update visualization",
          "timestamp": "2022-06-03T11:12:35-07:00",
          "tree_id": "ce7cfeaa18bc1d1b5751c5bb6c8cfb8b7bd5aa89",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/8ecbb2086c507476ffa2a60f51c50add4bfd43c9"
        },
        "date": 1654280299628,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 41810,
            "range": "± 329",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 920195,
            "range": "± 14486",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 13633030,
            "range": "± 162200",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 35091457,
            "range": "± 1712788",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 36584,
            "range": "± 209",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 838923,
            "range": "± 6540",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 12904389,
            "range": "± 79308",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 31839314,
            "range": "± 1911756",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "aa23f70a58bd764bf2a5efd548167758362f7811",
          "message": "DT: Add DiagonalTransition for linear costs, with+without linear memory",
          "timestamp": "2022-06-11T01:24:27-07:00",
          "tree_id": "9e98332e1047556fbe74630bd77d25e0cf568faf",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/aa23f70a58bd764bf2a5efd548167758362f7811"
        },
        "date": 1654936447650,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 50881,
            "range": "± 9197",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1133573,
            "range": "± 248895",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 47962637,
            "range": "± 7857000",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 104638281,
            "range": "± 12613086",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 40206,
            "range": "± 7660",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 982658,
            "range": "± 264430",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 42488197,
            "range": "± 6505541",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 84518848,
            "range": "± 20115889",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "92cdb685e25eeb5b9f33ef4c2a0c90ba8eef3dc9",
          "message": "AFFINE DIAGONAL TRANSITION; compiling",
          "timestamp": "2022-06-11T15:30:59-07:00",
          "tree_id": "a70bcd027abfcd775eda5bb36c8d0ce8ad030380",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/92cdb685e25eeb5b9f33ef4c2a0c90ba8eef3dc9"
        },
        "date": 1654986994233,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 40755,
            "range": "± 269",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1059937,
            "range": "± 15319",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 44614575,
            "range": "± 292104",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 89659946,
            "range": "± 1229342",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 37003,
            "range": "± 241",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 951544,
            "range": "± 11216",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 42448644,
            "range": "± 1122226",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 81879234,
            "range": "± 1261321",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "4663d1b0e8033186206a996c4867d49a1aad72fa",
          "message": "Make BruteforceCSH prune by arrow, not by start pos. And update pruning logic",
          "timestamp": "2022-06-11T20:55:32-07:00",
          "tree_id": "299050b1307662b3248125cac2ae73f07d2c6ad8",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/4663d1b0e8033186206a996c4867d49a1aad72fa"
        },
        "date": 1655006472590,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 35713,
            "range": "± 1392",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 883582,
            "range": "± 16108",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 38135733,
            "range": "± 728706",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 81949137,
            "range": "± 2972768",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 31427,
            "range": "± 552",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 777831,
            "range": "± 27043",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 35511048,
            "range": "± 596148",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 72129637,
            "range": "± 2920433",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "61ca386cd36ca9f979ef7b2d230b40c5794d2da5",
          "message": "DT: refactoring; cleanup; more comments",
          "timestamp": "2022-06-12T01:24:43-07:00",
          "tree_id": "ec0477527ffeed237dff03b8ba31144d43b08f81",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/61ca386cd36ca9f979ef7b2d230b40c5794d2da5"
        },
        "date": 1655022583957,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 42361,
            "range": "± 296",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1010723,
            "range": "± 11236",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 43126668,
            "range": "± 148353",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 86854061,
            "range": "± 1432207",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 35993,
            "range": "± 125",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 886581,
            "range": "± 2341",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 39922358,
            "range": "± 346899",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 77095393,
            "range": "± 1563930",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "7ddf763bc59eeb4e9d53b824efaa54e9d336eb5d",
          "message": "DT: Introduce `iota` array to map over to generate arrays",
          "timestamp": "2022-06-12T01:59:11-07:00",
          "tree_id": "ab60656e2cb97b22d7b52a5c76f8ebc4045c8d28",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/7ddf763bc59eeb4e9d53b824efaa54e9d336eb5d"
        },
        "date": 1655024739619,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 47860,
            "range": "± 7431",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1196780,
            "range": "± 85289",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 53611458,
            "range": "± 3724676",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 110282543,
            "range": "± 7477238",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 43150,
            "range": "± 7761",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 1058622,
            "range": "± 57495",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 49626877,
            "range": "± 3993394",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 99161034,
            "range": "± 4089592",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "285623af85483c47eb8b513c578813a2787f618b",
          "message": "DT: Significantly simplify code by not precomputing layers",
          "timestamp": "2022-06-12T02:12:25-07:00",
          "tree_id": "8da12d24285904a19fde8642fca71d6da8ea0862",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/285623af85483c47eb8b513c578813a2787f618b"
        },
        "date": 1655025445477,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 35307,
            "range": "± 930",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 885385,
            "range": "± 48888",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 38086581,
            "range": "± 496549",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 78517899,
            "range": "± 1944189",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 31438,
            "range": "± 917",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 766726,
            "range": "± 65455",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 35216044,
            "range": "± 473461",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 68675128,
            "range": "± 1798132",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "6ea2bef5ca3411b9ebbd5b1e3839c26955952b63",
          "message": "DT: Add GapClose optoin",
          "timestamp": "2022-06-12T02:41:00-07:00",
          "tree_id": "dfb12bf2736e785dbbcd3a510ba23ccf06b55d1b",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/6ea2bef5ca3411b9ebbd5b1e3839c26955952b63"
        },
        "date": 1655027256074,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 49994,
            "range": "± 5795",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1220197,
            "range": "± 255745",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 52356611,
            "range": "± 9502948",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 105795664,
            "range": "± 11980848",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 44417,
            "range": "± 6716",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 1011766,
            "range": "± 95212",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 45791992,
            "range": "± 4663331",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 96319761,
            "range": "± 8953070",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "a10c550a021ebda8303960a4d75b6ea31e2c22c9",
          "message": "DT: BiWFA overlap detection",
          "timestamp": "2022-06-12T15:21:22-07:00",
          "tree_id": "6a43d93e8cb6f7bc03dabdf76186b805da3e35d3",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/a10c550a021ebda8303960a4d75b6ea31e2c22c9"
        },
        "date": 1655072844976,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 46384,
            "range": "± 3714",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1121790,
            "range": "± 110364",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 47428008,
            "range": "± 3033257",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 99484790,
            "range": "± 8173819",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 40078,
            "range": "± 4390",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 979435,
            "range": "± 84148",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 44866170,
            "range": "± 4477133",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 88423363,
            "range": "± 5835794",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "dc79c272c76928659fdcbbde31d9fccf37698b55",
          "message": "DT: Use I/D symmetric version of DP that stores f.r. points as i+j instead of i",
          "timestamp": "2022-06-13T17:40:07-07:00",
          "tree_id": "6cd9953d76b97306f786390356da58dcaa25906a",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/dc79c272c76928659fdcbbde31d9fccf37698b55"
        },
        "date": 1655167523314,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 41633,
            "range": "± 259",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1003123,
            "range": "± 4682",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 43080548,
            "range": "± 1075302",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 87228654,
            "range": "± 2086621",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 35622,
            "range": "± 147",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 885401,
            "range": "± 2524",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 40070510,
            "range": "± 264471",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 78547972,
            "range": "± 2210188",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": false,
          "id": "225edee7c42e17825d5f3ec6c997db57ce1071f5",
          "message": "todo",
          "timestamp": "2022-06-16T00:24:02-07:00",
          "tree_id": "687c68006e437fd7096a159d2c4ebefcf3cb14ca",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/225edee7c42e17825d5f3ec6c997db57ce1071f5"
        },
        "date": 1655366533162,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 50093,
            "range": "± 2586",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1216675,
            "range": "± 20785",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 52199495,
            "range": "± 497454",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 107927024,
            "range": "± 1221815",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 44430,
            "range": "± 12766",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 1105286,
            "range": "± 242578",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 48377219,
            "range": "± 372346",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 97155991,
            "range": "± 1135556",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "63d0168f313f69da08c5620a6463d1139c876338",
          "message": "Merge pull request #2 from RagnarGrootKoerkamp/nw_path\n\nNw path",
          "timestamp": "2022-06-17T12:56:57-07:00",
          "tree_id": "b00a5a7244c090bca390bb196961c0d2bcc5b7ba",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/63d0168f313f69da08c5620a6463d1139c876338"
        },
        "date": 1655496221668,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 53184,
            "range": "± 9811",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1207502,
            "range": "± 128560",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 52492735,
            "range": "± 4252377",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 110534498,
            "range": "± 11661801",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 45343,
            "range": "± 5082",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 1043471,
            "range": "± 148471",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 46807432,
            "range": "± 6412064",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 96249787,
            "range": "± 7154613",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "225edee7c42e17825d5f3ec6c997db57ce1071f5",
          "message": "todo",
          "timestamp": "2022-06-16T00:24:02-07:00",
          "tree_id": "687c68006e437fd7096a159d2c4ebefcf3cb14ca",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/225edee7c42e17825d5f3ec6c997db57ce1071f5"
        },
        "date": 1655496725062,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 41563,
            "range": "± 512",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1004820,
            "range": "± 6835",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 43482309,
            "range": "± 464193",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 91025368,
            "range": "± 1752392",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 37090,
            "range": "± 360",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 885728,
            "range": "± 3506",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 41122499,
            "range": "± 308214",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 80803229,
            "range": "± 1372586",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "akulov5120@gmail.com",
            "name": "Mykola Akulov",
            "username": "mykola-akulov"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": false,
          "id": "612759dc083bde837a133bc945024a7a554cc335",
          "message": "Fixed test",
          "timestamp": "2022-06-17T13:04:45-07:00",
          "tree_id": "b00a5a7244c090bca390bb196961c0d2bcc5b7ba",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/612759dc083bde837a133bc945024a7a554cc335"
        },
        "date": 1655496880431,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 44411,
            "range": "± 5031",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1026247,
            "range": "± 90998",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 44671377,
            "range": "± 4448383",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 90051932,
            "range": "± 8130751",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 39524,
            "range": "± 7983",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 875393,
            "range": "± 110384",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 41736334,
            "range": "± 4532576",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 75957019,
            "range": "± 4861218",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "2f63eb5a104f2493150d25225b37c6da9d121d6a",
          "message": "clean up test",
          "timestamp": "2022-06-17T13:20:24-07:00",
          "tree_id": "ac19b20857dd1bf31b53f786e85102a0f731a758",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/2f63eb5a104f2493150d25225b37c6da9d121d6a"
        },
        "date": 1655497614719,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 52667,
            "range": "± 11269",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1155654,
            "range": "± 160997",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 51276556,
            "range": "± 5011944",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 105414404,
            "range": "± 10796558",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 42930,
            "range": "± 7297",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 1022128,
            "range": "± 176416",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 44139934,
            "range": "± 4598299",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 90444921,
            "range": "± 12211008",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "66d7de31b72e8a478142573c5ff965730aa26951",
          "message": "NT/DT: cleanup",
          "timestamp": "2022-06-17T23:45:45-07:00",
          "tree_id": "b416d70c115c9cda3e89ede6ac9399550a29813d",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/66d7de31b72e8a478142573c5ff965730aa26951"
        },
        "date": 1655538021524,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 42174,
            "range": "± 1689",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1054473,
            "range": "± 12776",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 44551932,
            "range": "± 268145",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 89525635,
            "range": "± 928689",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 37591,
            "range": "± 269",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 955529,
            "range": "± 10514",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 42299201,
            "range": "± 790539",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 81361716,
            "range": "± 988587",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "92dc6a5630a3adb193955fd497e73a03a23ffe38",
          "message": "Front: Inline Layers into Front; add {new,reset}[_with_buffer] convenience functions",
          "timestamp": "2022-06-22T22:11:27+02:00",
          "tree_id": "78d87c85aae0bfaf1b2ba98d2c5efc46ae8a68db",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/92dc6a5630a3adb193955fd497e73a03a23ffe38"
        },
        "date": 1655929042620,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 41481,
            "range": "± 338",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1054112,
            "range": "± 11344",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 45224302,
            "range": "± 205579",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 90648836,
            "range": "± 497169",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 37738,
            "range": "± 261",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 949691,
            "range": "± 14122",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 41851885,
            "range": "± 199621",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 81022069,
            "range": "± 636885",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "3db0fc94e6407ff8631c7315a137613cb8877085",
          "message": "ExpBand: Fix many bugs. WORKING NOW",
          "timestamp": "2022-06-23T04:45:41+02:00",
          "tree_id": "3b72b6b1bf3e5b9b3b0907c8d1f9532c9c975443",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/3db0fc94e6407ff8631c7315a137613cb8877085"
        },
        "date": 1655952669699,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 44950,
            "range": "± 344",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1058624,
            "range": "± 14515",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 44658323,
            "range": "± 383574",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 89255235,
            "range": "± 1288412",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 37145,
            "range": "± 227",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 944393,
            "range": "± 10547",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 42538757,
            "range": "± 1849411",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 80738837,
            "range": "± 1767779",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "3c0b6be0c9a62a79dceb2b9b415ff92402b51a44",
          "message": "DT: Use Fronts",
          "timestamp": "2022-06-24T08:47:46+02:00",
          "tree_id": "b89b0f3a18d2684082cb70b5232cbecfaf73ca43",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/3c0b6be0c9a62a79dceb2b9b415ff92402b51a44"
        },
        "date": 1656053649027,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 50536,
            "range": "± 396",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1264952,
            "range": "± 1495377",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 53988526,
            "range": "± 680725",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 110605490,
            "range": "± 1887281",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 45485,
            "range": "± 344",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 1109519,
            "range": "± 4491",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 50970516,
            "range": "± 407223",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 99672011,
            "range": "± 2482375",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "eae6659589541a371efcd53f2505d4cd8e3c2d2e",
          "message": "Bench: Add benchmark for NW/DT aligners with/out exp search and gap-heuristic",
          "timestamp": "2022-06-24T12:55:06+02:00",
          "tree_id": "e70835faf03e0980710449efac3ff65f3e7bce52",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/eae6659589541a371efcd53f2505d4cd8e3c2d2e"
        },
        "date": 1656068514775,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 47825,
            "range": "± 1712",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1160565,
            "range": "± 49365",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 50094651,
            "range": "± 1694589",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 100420654,
            "range": "± 8513654",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 40867,
            "range": "± 3022",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 1014612,
            "range": "± 80423",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 44945715,
            "range": "± 3064503",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 92764142,
            "range": "± 7504964",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "a36ea2c99fbac1a2fdecf8ae41d069f73524e867",
          "message": "Add some utils: Pos::root, CostModel::linear_cost_in_direction",
          "timestamp": "2022-06-25T10:24:53+02:00",
          "tree_id": "2b29bf07efafc1e40db82077f19f198062b2b4be",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/a36ea2c99fbac1a2fdecf8ae41d069f73524e867"
        },
        "date": 1656147193496,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 36406,
            "range": "± 1033",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 891821,
            "range": "± 15781",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 38933161,
            "range": "± 541559",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 80919355,
            "range": "± 3560118",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 31694,
            "range": "± 245",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 782768,
            "range": "± 26044",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 40416557,
            "range": "± 5341164",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 79981315,
            "range": "± 2964885",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "c56e665b474eb0d6c1030df74bd3b41e69cd3cb5",
          "message": "DT: Use new EditGraph function",
          "timestamp": "2022-06-25T23:24:01+02:00",
          "tree_id": "ccfecade237e4c8ffa57e90747254c89bfd40c94",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/c56e665b474eb0d6c1030df74bd3b41e69cd3cb5"
        },
        "date": 1656192726785,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 41456,
            "range": "± 467",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1008962,
            "range": "± 6868",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 43598236,
            "range": "± 408174",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 89221637,
            "range": "± 1976723",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 36121,
            "range": "± 337",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 885960,
            "range": "± 4111",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 40264999,
            "range": "± 473273",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 78095791,
            "range": "± 1821825",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "6a804805666d1ba01426215d58defba65d270e3f",
          "message": "DIAGONAL TRANSITION + EXPONENTIAL SEARCH + HEURISTIC",
          "timestamp": "2022-06-26T11:17:19+02:00",
          "tree_id": "0b3eccdef3f272077d69d69a1831dbcc9ffd8a16",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/6a804805666d1ba01426215d58defba65d270e3f"
        },
        "date": 1656235342708,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 49438,
            "range": "± 745",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1057227,
            "range": "± 1883",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 38667745,
            "range": "± 678900",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 93121022,
            "range": "± 2696519",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 41172,
            "range": "± 464",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 924014,
            "range": "± 2901",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 37634974,
            "range": "± 614548",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 84283018,
            "range": "± 2818426",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "eff333967ef8ee0a57e5b56f05cf43b9c4ae21cc",
          "message": "fix bad include warning",
          "timestamp": "2022-07-11T11:12:47+02:00",
          "tree_id": "4b55833b56b4e88da7e1a1212372886aa6bede86",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/eff333967ef8ee0a57e5b56f05cf43b9c4ae21cc"
        },
        "date": 1657531015911,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 41844,
            "range": "± 299",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 904350,
            "range": "± 10041",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 13261103,
            "range": "± 81432",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 34300089,
            "range": "± 993697",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 37954,
            "range": "± 416",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 825403,
            "range": "± 8114",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 12764355,
            "range": "± 95831",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 31020653,
            "range": "± 2336566",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "akulov5120@gmail.com",
            "name": "Mykola Akulov",
            "username": "mykola-akulov"
          },
          "committer": {
            "email": "akulov5120@gmail.com",
            "name": "Mykola Akulov",
            "username": "mykola-akulov"
          },
          "distinct": true,
          "id": "c6e4b2ae44038d5383b4d03fe2a27a785a2f3f3e",
          "message": "layered drawing for NW",
          "timestamp": "2022-07-11T12:22:45+03:00",
          "tree_id": "ce69df43372188e81592146353f04e792f3c1abf",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/c6e4b2ae44038d5383b4d03fe2a27a785a2f3f3e"
        },
        "date": 1657531792110,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 39798,
            "range": "± 622",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 878542,
            "range": "± 11134",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 12677368,
            "range": "± 183409",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 35615223,
            "range": "± 2928660",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 35384,
            "range": "± 259",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 786579,
            "range": "± 3615",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 11715155,
            "range": "± 130539",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 32041807,
            "range": "± 2453524",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "1a50b6373bae383ada6d27c5c0d4feb963b1c070",
          "message": "Fix tests in --no-default-dependencies mode",
          "timestamp": "2022-07-11T11:40:26+02:00",
          "tree_id": "c92c3a5c321e470ee49c30226409070d058fd600",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/1a50b6373bae383ada6d27c5c0d4feb963b1c070"
        },
        "date": 1657532698473,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 39892,
            "range": "± 465",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 876688,
            "range": "± 4207",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 12598821,
            "range": "± 21057",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 32537598,
            "range": "± 283808",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 35372,
            "range": "± 229",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 786287,
            "range": "± 3354",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 11650461,
            "range": "± 22282",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 28470764,
            "range": "± 557673",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "97878da0a25f2d3b859b6e7b36021d5cf65eea8e",
          "message": "Fix test: do not call CostModel::extend with decreasing position",
          "timestamp": "2022-07-11T12:02:58+02:00",
          "tree_id": "be73ca87277fcc609f2697074e61bb7fbd7b2447",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/97878da0a25f2d3b859b6e7b36021d5cf65eea8e"
        },
        "date": 1657534055127,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 40061,
            "range": "± 300",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 877671,
            "range": "± 6506",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 12620680,
            "range": "± 39141",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 33354493,
            "range": "± 2163995",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 35462,
            "range": "± 145",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 784919,
            "range": "± 3028",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 11742778,
            "range": "± 166356",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 29379382,
            "range": "± 2289643",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "d3f920b946cadcccb2129fe35219ba89cb7d623b",
          "message": "Tests: Add linear cost breaking case for DT",
          "timestamp": "2022-07-11T14:40:10+02:00",
          "tree_id": "b404e3c92bf0a72da98e3a7d814773fef2fee537",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/d3f920b946cadcccb2129fe35219ba89cb7d623b"
        },
        "date": 1657543459581,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 35162,
            "range": "± 169",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 771254,
            "range": "± 7510",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 11195018,
            "range": "± 68467",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 29402275,
            "range": "± 1093919",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 31033,
            "range": "± 192",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 693991,
            "range": "± 5144",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 10351719,
            "range": "± 37778",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 25783484,
            "range": "± 1367717",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "6a1a531f143aff53373793e2c7c52ce0ca576399",
          "message": "Small cleanup",
          "timestamp": "2022-07-11T21:49:53+02:00",
          "tree_id": "4ff4c80326da50dfad65adf9c44246b91d3a50c6",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/6a1a531f143aff53373793e2c7c52ce0ca576399"
        },
        "date": 1657569236665,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 40675,
            "range": "± 192",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 875737,
            "range": "± 9669",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 12724789,
            "range": "± 348336",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 35659287,
            "range": "± 4351015",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 35291,
            "range": "± 155",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 787165,
            "range": "± 3202",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 11716292,
            "range": "± 176312",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 30802177,
            "range": "± 3925945",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "2e03958726e6fa774fc0034f3d40919b635580ac",
          "message": "Edlib: Fix build.rs",
          "timestamp": "2022-07-11T22:35:37+02:00",
          "tree_id": "8b6468a28bee19866391b366b27e8a54a3c70adb",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/2e03958726e6fa774fc0034f3d40919b635580ac"
        },
        "date": 1657571981572,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 42251,
            "range": "± 2283",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 905479,
            "range": "± 18028",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 13450607,
            "range": "± 168849",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 37762144,
            "range": "± 1259749",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 37691,
            "range": "± 297",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 828073,
            "range": "± 9434",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 12920482,
            "range": "± 546916",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 34133062,
            "range": "± 1725540",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "62757ea2138e24611b694410398db7cb486dc181",
          "message": "Tests: fix build errors",
          "timestamp": "2022-07-11T22:39:22+02:00",
          "tree_id": "d34dc83a3b558a01084142dc83004fc705587e42",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/62757ea2138e24611b694410398db7cb486dc181"
        },
        "date": 1657572209272,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 42542,
            "range": "± 4508",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 908508,
            "range": "± 13364",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 13237820,
            "range": "± 69632",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 34637974,
            "range": "± 2391051",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 37728,
            "range": "± 193",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 829713,
            "range": "± 6611",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 12733359,
            "range": "± 42040",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 30510158,
            "range": "± 1666425",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "a33c18662dc5e68c6d7f4af0cd792d2dc64ba80e",
          "message": "Tests: enable exponential_search with SH",
          "timestamp": "2022-07-11T22:44:24+02:00",
          "tree_id": "3ce30befe5ba78de646726f9ffc0075ef56d64bb",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/a33c18662dc5e68c6d7f4af0cd792d2dc64ba80e"
        },
        "date": 1657572513411,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 41987,
            "range": "± 169",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 906620,
            "range": "± 13030",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 13358950,
            "range": "± 162783",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 36179835,
            "range": "± 1883886",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 37579,
            "range": "± 202",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 830045,
            "range": "± 12658",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 12750431,
            "range": "± 122183",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 31815514,
            "range": "± 1373703",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "2821f8c4ab564e0f079cb676278cd2f18b22206a",
          "message": "CI: Use rust cache action",
          "timestamp": "2022-07-11T22:51:57+02:00",
          "tree_id": "ec7651b188d553385277d7dc8f55b3a69fe73286",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/2821f8c4ab564e0f079cb676278cd2f18b22206a"
        },
        "date": 1657573036138,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 49501,
            "range": "± 11668",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1062994,
            "range": "± 142530",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 14883797,
            "range": "± 1750099",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 43686632,
            "range": "± 3773134",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 44530,
            "range": "± 4271",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 878751,
            "range": "± 232216",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 13953723,
            "range": "± 1546502",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 39248176,
            "range": "± 3561068",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "yurii.ketkov@gmail.com",
            "name": "ketkovyurii",
            "username": "ketkovyurii"
          },
          "committer": {
            "email": "yurii.ketkov@gmail.com",
            "name": "ketkovyurii",
            "username": "ketkovyurii"
          },
          "distinct": true,
          "id": "dd31d2baaa4f38e42f03247994de0f4c3633705c",
          "message": "biwfa changes",
          "timestamp": "2022-07-12T13:53:03+02:00",
          "tree_id": "ddf48566f3168f14f11bd157b6c5cf79054b6aa8",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/dd31d2baaa4f38e42f03247994de0f4c3633705c"
        },
        "date": 1657627028858,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 39599,
            "range": "± 575",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 865185,
            "range": "± 5339",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 12652833,
            "range": "± 259069",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 33928997,
            "range": "± 3117794",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 35276,
            "range": "± 114",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 775073,
            "range": "± 2859",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 11487066,
            "range": "± 161820",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 31079244,
            "range": "± 1702970",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "2821f8c4ab564e0f079cb676278cd2f18b22206a",
          "message": "CI: Use rust cache action",
          "timestamp": "2022-07-11T22:51:57+02:00",
          "tree_id": "ec7651b188d553385277d7dc8f55b3a69fe73286",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/2821f8c4ab564e0f079cb676278cd2f18b22206a"
        },
        "date": 1657627303466,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 40259,
            "range": "± 323",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 866205,
            "range": "± 4882",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 12585042,
            "range": "± 335283",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 35305354,
            "range": "± 2593671",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 34892,
            "range": "± 125",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 776115,
            "range": "± 2228",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 11494134,
            "range": "± 110744",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 30570507,
            "range": "± 3021503",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "yurii.ketkov@gmail.com",
            "name": "ketkovyurii",
            "username": "ketkovyurii"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "2258a72822f20542e7caafc4cb7a51218e0ad968",
          "message": "biwfa tests",
          "timestamp": "2022-07-12T15:29:26+02:00",
          "tree_id": "bfae63c190cdafd10f5d08e5028f7dc5922ea487",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/2258a72822f20542e7caafc4cb7a51218e0ad968"
        },
        "date": 1657632720445,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 44715,
            "range": "± 3303",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 885073,
            "range": "± 91994",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 12940580,
            "range": "± 1797627",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 36275751,
            "range": "± 10364599",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 38856,
            "range": "± 13071",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 834008,
            "range": "± 183575",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 11785266,
            "range": "± 1331948",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 30467297,
            "range": "± 5403908",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "5f9ae15a2378b5e4145782c9bdc335f36fbc681a",
          "message": "DT: ENABLE TESTS; ALL PASSING!",
          "timestamp": "2022-07-13T01:51:02+02:00",
          "tree_id": "db87e78d77ce8ebdc871f4321c64b8705ef57b73",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/5f9ae15a2378b5e4145782c9bdc335f36fbc681a"
        },
        "date": 1657670062213,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 40895,
            "range": "± 443",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 868150,
            "range": "± 4475",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 12664131,
            "range": "± 199121",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 35403961,
            "range": "± 1605101",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 35549,
            "range": "± 201",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 778247,
            "range": "± 2595",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 11596176,
            "range": "± 124221",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 30131830,
            "range": "± 1363540",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "ef1bfeb77816bc586fdbdfd14335ff31d9083db9",
          "message": "DT: Add some more tests; fix possible underflow",
          "timestamp": "2022-07-13T02:02:07+02:00",
          "tree_id": "e49a690be372c3645f2f96874c4e4a716e10bc9e",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/ef1bfeb77816bc586fdbdfd14335ff31d9083db9"
        },
        "date": 1657670657943,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 40706,
            "range": "± 220",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 868792,
            "range": "± 3810",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 12582080,
            "range": "± 28661",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 32513413,
            "range": "± 489026",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 35486,
            "range": "± 161",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 779820,
            "range": "± 2956",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 11527998,
            "range": "± 37712",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 27987298,
            "range": "± 848895",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "207a3dacb80f34131eb0909503aa71d99a59e0d7",
          "message": "CI: Run tests in --release mode for speed",
          "timestamp": "2022-07-13T02:23:47+02:00",
          "tree_id": "23715a3e5f1f0e8c32887e2f8d1a22b232c51dd8",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/207a3dacb80f34131eb0909503aa71d99a59e0d7"
        },
        "date": 1657672011740,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 48953,
            "range": "± 10432",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 1059263,
            "range": "± 97602",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 15703799,
            "range": "± 2193627",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 45736657,
            "range": "± 8475708",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 43524,
            "range": "± 4678",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 928693,
            "range": "± 157833",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 13490334,
            "range": "± 1969662",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 41058735,
            "range": "± 5972736",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "committer": {
            "email": "ragnar.grootkoerkamp@gmail.com",
            "name": "Ragnar Groot Koerkamp",
            "username": "RagnarGrootKoerkamp"
          },
          "distinct": true,
          "id": "d528c2ba500a5a4cef2e8f5e14401f5675f7cbc4",
          "message": "WFA: Free memory; fix signs of returned scores",
          "timestamp": "2022-07-13T02:46:42+02:00",
          "tree_id": "0a8c40ff2ab6498630e01da2995c032af9f7faf3",
          "url": "https://github.com/RagnarGrootKoerkamp/astar-pairwise-aligner/commit/d528c2ba500a5a4cef2e8f5e14401f5675f7cbc4"
        },
        "date": 1657673340359,
        "tool": "cargo",
        "benches": [
          {
            "name": "base_100",
            "value": 40020,
            "range": "± 194",
            "unit": "ns/iter"
          },
          {
            "name": "base_1000",
            "value": 887504,
            "range": "± 11697",
            "unit": "ns/iter"
          },
          {
            "name": "base_10000",
            "value": 13219950,
            "range": "± 271592",
            "unit": "ns/iter"
          },
          {
            "name": "base_50000_similar",
            "value": 35107124,
            "range": "± 2353429",
            "unit": "ns/iter"
          },
          {
            "name": "fast_100",
            "value": 35854,
            "range": "± 94",
            "unit": "ns/iter"
          },
          {
            "name": "fast_1000",
            "value": 809946,
            "range": "± 10720",
            "unit": "ns/iter"
          },
          {
            "name": "fast_10000",
            "value": 12642193,
            "range": "± 314580",
            "unit": "ns/iter"
          },
          {
            "name": "fast_50000_similar",
            "value": 30711948,
            "range": "± 2225893",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}