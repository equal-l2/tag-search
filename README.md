# tag-search
情報科学実験I超高性能化課題のための検索サーバ  
(オプションでキャッシュあり、通常無効なので、有効にするにはビルド時に--with-featuresでcacheを有効にする)    

    $ tag-search <tag_pp.csv/geotag_pp.csvのあるディレクトリ>


サーバの仕様:  
- ポート番号は8080番  
- `/query.html`でリクエストを受け付ける  
- クエリパラメータは次の2つ  
    - `tag`   (文字列): タグ  
    - `cache` (真偽値): キャッシュを有効にするか  

関連クレート
- [tag-pp](https://github.com/equal-l2/tag-pp): データ前処理
- [tag-geotag](https://github.com/equal-l2/tag-geotag): データ構造
