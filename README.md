# tag-search
情報科学実験I超高性能化課題のための検索サーバ  
(オプションでキャッシュあり、通常無効なので、有効にするにはビルド時に--with-featuresでcacheを有効にする)    

    $ tag-search <tag_pp.csv/geotag_pp.csvのあるディレクトリ>

## コンパイルの仕方
これと `tag-geotag` を同じフォルダにクローン(下記参照)してから、Cargoでビルド
```
適当なフォルダ/
├── tag-geotag/
└── tag-search/
```

## サーバの仕様:  
- ポート番号は8080番  
- `/query.html`でリクエストを受け付ける  
- クエリパラメータは次の2つ  
    - `tag`   (文字列): タグ  
    - `cache` (真偽値): キャッシュを有効にするか  

## 関連クレート
- [tag-pp](https://github.com/equal-l2/tag-pp): データ前処理
- [tag-geotag](https://github.com/equal-l2/tag-geotag): データ構造

## おまけ
[進捗をつぶやいた文章をQiitaの下書きから発掘したのでおいておきます](https://gist.github.com/equal-l2/afda48a947e9c3b6d0c9413a663fd812)  
最適化の試行錯誤の過程が多少書いてあるので役に立つかも？
