# recent-backend-investigations

Findy社の寄稿用に最近開発が進むRustのバックエンドのアプリケーションの調査を行ったリポジトリです。Poem、Loco、CotでそれぞれToDoアプリを実装し、どのような開発体験が提供されているかを簡単に知ることを目的としています。

## 環境構築

前提として、データベースにPostgreSQLを使用しています。アプリケーションを起動する前に、事前にDockerで立ち上げておく必要があります。

```
docker compose up -d
```

また、Poemの事例については`psqldef`の利用を前提としたデータベースマイグレーションを行います。

> ![NOTE]
> `psqldef`のインストールや概要については、こちらのページをご覧ください。
> https://github.com/sqldef/sqldef

データベースマイグレーションは下記のコマンドで実行できます。

```
psqldef -U postgres -W password todo_app < migration/v0.sql
```

LocoとCotは、そもそもフレームワーク側にマイグレーション機能が用意されているため、それを利用します。

## それぞれのアプリケーションの起動

### Poemの起動

```
cargo run -p todo-poem
```

`localhost:8081`にサーバーが起動します。

### Locoの起動

まず、データベースマイグレーションを実行します。

```
# todo_loco
cargo loco db migrate
```

次に、サーバーを起動します。

```
# todo_loco
cargo loco start
```

`localhost:5150`にサーバーが起動します。

### Cotの起動

まず、データベースマイグレーションを実行します。

```
# todo_cot
cot migration make
```

```
# todo_cot
cargo run
```

`localhost:8000`にサーバーが起動します。

## 非同期ランタイムのバイナリサイズ

tokioとsmolでそれぞれほとんど同じようなコードを用意し、バイナリサイズを比較してみました。smolの方が小さくなる傾向にはありそうですが、smolを使っている限りは劇的に差がある感じもしません。コードの内容によっては大きく差が出てるケースもあるかもしれません。

```
cd runtime
cargo build --bin tokio --release
cargo build --bin smol --release
```

```
Size Date Modified Name
732k 15 Jul 00:08   smol
 185 15 Jul 00:08   smol.d
881k 15 Jul 00:08   tokio
 187 15 Jul 00:08   tokio.d
```
