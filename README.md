hone
====

```console
$ hone run -- bash -c 'foo.py lr=`hone get float --ln x 0.0001 1.0` | grep -oP "(?<=ACC:)[0-9.]*" | hone report'
```

```console
$ hone init

$ hone run --study-name foo -- bash 'foo.py lr=`hone get x int 0 10`'
$ hone study foo-bar
$ for i in `seq 1 10`
$ then
$   hone start
$   X=`hone get x int 0 10`
$   Y=`hone get y int 3 8`
$   echo $(($X * $Y)) | hone observe
$   hone end
$ end

$ hone show best-params
```

Directory Structure
-------------------

```
.hone/
  config.json
  data/
    {STUDY_ID}/
      finished/
      running/
        {THREAD_ID}/
```

Distributed Optimization
------------------------

You can parallelize your optimization process using ordinal unix functionalities:
- NFS

[MEMO]
Unlike Optuna, hone doesn't support RDB storage. Instead, it allows data to be shared via NFS ...
