<div align="center">

![Mania](https://socialify.git.ci/LagrangeDev/mania/image?description=1&descriptionEditable=An%20Implementation%20of%20NTQQ%20Protocol,%20with%20Pure%20Rust%F0%9F%A6%80,%20Derived%20from%20Lagrange.Core&font=Jost&forks=1&issues=1&logo=https%3A%2F%2Fstatic.live.moe%2Flagrange.jpg&name=1&pattern=Diagonal%20Stripes&pulls=1&stargazers=1&theme=Auto)
[![GitHub Workflow Status (with event)](https://img.shields.io/github/actions/workflow/status/LagrangeDev/mania/check.yml?logo=github)](https://github.com/LagrangeDev/mania/actions)
![nightly](https://img.shields.io/badge/toolchain-nightly-important)
![wip](https://img.shields.io/badge/develop-wip-blue)

</div>

> [!NOTE]\
> This project is originally frozen in
> [radicle](https://app.radicle.xyz/nodes/seed.radicle.garden/rad:z4QZVPDxLbGgd1oHFsjtJLQYtZ8ma)

> [!WARNING]\
> This project is still in active development. The internal and external
> interfaces are still unstable

## Disclaimer

[See Lagrange.Core](https://github.com/LagrangeDev/Lagrange.Core#disclaimer)

## Features List

| Protocol | Support | Login                          | Support | Messages      | Support | Operations        | Support | Events              | Support |
| -------- | :-----: | ------------------------------ | :-----: | :------------ | :-----: | :---------------- | :-----: | :------------------ | :-----: |
| Windows  |   🔴    | QrCode                         |   🟢    | BounceFace    |   🔴    | Poke              |   🔴    | ~~Captcha~~         |   🔴    |
| macOS    |   🔴    | ~~Password~~                   |   🔴    | Face          | 🟡 [^1] | Recall            |   🔴    | BotOnline           |   🟢    |
| Linux    |   🟢    | EasyLogin                      |   🟡    | File          | 🟡[^1]  | Leave Group       |   🔴    | BotOffline          |   🟢    |
|          |         | ~~UnusualDevice<br/>Password~~ |   🔴    | Forward       |   🟢    | Set Special Title |   🔴    | Message             |   🟢    |
|          |         | ~~UnusualDevice<br/>Easy~~     |   🔴    | ~~GreyTip~~   |   🔴    | Kick Member       |   🔴    | Poke                |   🟢    |
|          |         | ~~NewDeviceVerify~~            |   🔴    | GroupReaction | 🟡[^1]  | Mute Member       |   🔴    | MessageRecall       |   🟢    |
|          |         |                                |         | Image         |   🟢    | Set Admin         |   🔴    | GroupMemberDecrease |   🟢    |
|          |         |                                |         | Json          |   🟢    | Friend Request    |   🔴    | GroupMemberIncrease |   🟢    |
|          |         |                                |         | KeyBoard      |   🔴    | Group Request     |   🔴    | GroupPromoteAdmin   |   🟢    |
|          |         |                                |         | LightApp      |   🟢    | ~~Voice Call~~    |   🔴    | GroupInvite         |   🟢    |
|          |         |                                |         | LongMsg       | 🟡[^1]  | Client Key        |   🔴    | GroupRequestJoin    |   🟢    |
|          |         |                                |         | Markdown      |   🔴    | Cookies           |   🔴    | FriendRequest       |   🟢    |
|          |         |                                |         | MarketFace    | 🟡[^1]  | Send Message      |   🟡    | ~~FriendTyping~~    |   🔴    |
|          |         |                                |         | Mention       |   🟢    |                   |         | ~~FriendVoiceCall~~ |   🔴    |
|          |         |                                |         | MultiMsg      | 🟡[^1]  |                   |         |                     |         |
|          |         |                                |         | Poke          |   🔴    |                   |         |                     |         |
|          |         |                                |         | Record        |   🟢    |                   |         |                     |         |
|          |         |                                |         | SpecialPoke   |   🔴    |                   |         |                     |         |
|          |         |                                |         | Text          |   🟢    |                   |         |                     |         |
|          |         |                                |         | Video         |   🟢    |                   |         |                     |         |
|          |         |                                |         | Xml           |   🟢    |                   |         |                     |         |

[^1]: Only implemented event parsing

## Development

Make sure to set up the Git hooks before contributing by moving the hook files
under `scripts` directory to `.git/hooks`:

```bash
mkdir -p .git/hooks
cp scripts/pre-commit .git/hooks/pre-commit
cp scripts/pre-push .git/hooks/pre-push
```

## References

- All projects in [LagrangeDev](https://github.com/lagrangeDev) (and their twin
  projects)
- [lz1998/ricq](https://github.com/lz1998/ricq)
- [inmes-dev/qqbot.rs](https://github.com/inmes-dev/qqbot.rs)
