# tweet-like-listener (推特点赞图片下载)

号养好了，每天上推特都是色图，又懒得手动右键下载图片，那就把这项工作自动化吧！

## 功能
* 指定账号轮询，下载其近期点赞的推特图片到本地
* 多账号支持，群 友 严 选

## 使用
1. **important** 需要[推特开发者账号](https://developer.twitter.com/en/portal/petition/essential/basic-info)，请在里面申请并创建一个应用，得到 access_key（这是你访问推特 API 的凭证）
2. **important** 自行解决科学上网问题
3. release 下载对应平台应用（也可以本地 cargo 编译）
4. 执行应用（第一次执行会生成 config.toml 文件，如果没有请在相同目录下新建）
5. 填写 config.toml 文件
6. 再次执行应用，并使用任意方式让其能一直跑下去

## 注意事项
* 下载图片命名规则：
```
yyyy-mm-dd.name.@username.tweet_id.idx.(jpg|png)
```
如 `2022-10-21.ASK.@askziye.1583434009488420865.0.jpg` 表示 ASK 老师在 2022-10-21 的某条推特的第一张图 (0-indexed)，链接为`https://twitter.com/askziye/status/1583434009488420865`

* 懒得做分页，轮询一次最多五十条点赞，请按自己需求控制轮询频率
