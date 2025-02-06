<div align="center">

![Mania](https://socialify.git.ci/LagrangeDev/mania/image?description=1&descriptionEditable=An%20Implementation%20of%20NTQQ%20Protocol,%20with%20Pure%20Rust%F0%9F%A6%80,%20Derived%20from%20Lagrange.Core&font=Jost&forks=1&issues=1&logo=https%3A%2F%2Fstatic.live.moe%2Flagrange.jpg&name=1&pattern=Diagonal%20Stripes&pulls=1&stargazers=1&theme=Auto)
![nightly](https://img.shields.io/badge/toolchain-nightly-important)
![wip](https://img.shields.io/badge/develop-wip-blue)

</div>

> [!NOTE]\
> This project is originally frozen in [radicle](https://app.radicle.xyz/nodes/seed.radicle.garden/rad:z4QZVPDxLbGgd1oHFsjtJLQYtZ8ma)


> [!WARNING]  
> This project is still in active development. The internal and external interfaces are still unstable


## Disclaimer
[See Lagrange.Core](https://github.com/LagrangeDev/Lagrange.Core#disclaimer)

## Features List

| Protocol | Support | Login                      | Support | Messages      | Support | Operations        | Support | Events              | Support |
|----------|:-------:|----------------------------|:-------:|:--------------|:-------:|:------------------|:-------:|:--------------------|:-------:|
| Windows  |   🔴    | QrCode                     |   🟢    | BounceFace    |   🔴    | Poke              |   🔴    | Captcha             |   🔴    |
| macOS    |   🔴    | ~~Password~~               |   🔴    | Face          | 🟡 [^1] | Recall            |   🔴    | BotOnline           |   🔴    |
| Linux    |   🟢    | EasyLogin                  |   🟡    | File          | 🟡[^1]  | Leave Group       |   🔴    | BotOffline          |   🔴    |
|          |         | UnusualDevice<br/>Password |   🔴    | Forward       | 🟡[^1]  | Set Special Title |   🔴    | Message             |   🔴    |
|          |         | UnusualDevice<br/>Easy     |   🔴    | ~~GreyTip~~   |   🔴    | Kick Member       |   🔴    | Poke                |   🔴    |
|          |         | ~~NewDeviceVerify~~        |   🔴    | GroupReaction |   🔴    | Mute Member       |   🔴    | MessageRecall       |   🔴    |
|          |         |                            |         | Image         | 🟡[^1]  | Set Admin         |   🔴    | GroupMemberDecrease |   🔴    |
|          |         |                            |         | Json          |   🟢    | Friend Request    |   🔴    | GroupMemberIncrease |   🔴    |
|          |         |                            |         | KeyBoard      |   🔴    | Group Request     |   🔴    | GroupPromoteAdmin   |   🔴    |
|          |         |                            |         | LightApp      | 🟡[^1]  | ~~Voice Call~~    |   🔴    | GroupInvite         |   🔴    |
|          |         |                            |         | LongMsg       |   🔴    | Client Key        |   🔴    | GroupRequestJoin    |   🔴    |
|          |         |                            |         | Markdown      |   🔴    | Cookies           |   🔴    | FriendRequest       |   🔴    |
|          |         |                            |         | MarketFace    | 🟡[^1]  | Send Message      |   🔴    | ~~FriendTyping~~    |   🔴    |
|          |         |                            |         | Mention       | 🟡[^1]  |                   |         | ~~FriendVoiceCall~~ |   🔴    |
|          |         |                            |         | MultiMsg      | 🟡[^1]  |                   |         |                     |         |
|          |         |                            |         | Poke          |   🔴    |                   |         |                     |         |
|          |         |                            |         | Record        | 🟡[^1]  |                   |         |                     |         |
|          |         |                            |         | SpecialPoke   |   🔴    |                   |         |                     |         |
|          |         |                            |         | Text          |   🟢    |                   |         |                     |         |
|          |         |                            |         | Video         |   🔴    |                   |         |                     |         |
|          |         |                            |         | Xml           |   🔴    |                   |         |                     |         |

[^1]: Only implemented event parsing

## References
- All projects in [LagrangeDev](https://github.com/lagrangeDev) (and their twin projects)
- [lz1998/ricq](https://github.com/lz1998/ricq)
- [inmes-dev/qqbot.rs](https://github.com/inmes-dev/qqbot.rs)