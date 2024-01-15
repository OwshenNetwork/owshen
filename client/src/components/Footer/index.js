import TwitterIcon from "../../pics/twitter.png";
import GithubIcon from "../../pics/github.png";
import DiscordIcon from "../../pics/discord.png";

const Footer = () => {
  return (
    <footer className="mt-auto flex  mx-auto ">
      <a href="https://twitter.com/OweshenNetwork">
        <img src={TwitterIcon} className="w-8 mx-4" />
      </a>
      <a href="https://discord.gg/owshen">
        <img src={DiscordIcon} className="mx-4 w-8" />
      </a>
      <a href="https://github.com/OwshenNetwork/owshen">
        <img src={GithubIcon} className="mx-4 w-8" />
      </a>
    </footer>
  );
};

export default Footer;
