{ config, withSystem, ... }:
let
  inherit (withSystem "x86_64-linux" ({ config, ... }: config.site)) name dockerImage composeProjectFile;
in
{
  herculesCI = herculesCI: {
    onPush.default.outputs.effects.deploy = withSystem config.defaultEffectSystem (
      { pkgs, hci-effects, ... }:
      hci-effects.runIf (herculesCI.config.repo.branch == "main") (
        hci-effects.mkEffect {
          secretsMap.deploy-key = "deploy-key";
          userSetupScript = ''
            OLD_UMASK=$(umask)
            umask 377
            readSecretString deploy-key .privateKey >~/.ssh/deploy
            umask "$OLD_UMASK"
            DEPLOY_HOST="$(readSecretString deploy-key .host)"
            DEPLOY_USER="$(readSecretString deploy-key .user)"
            cat << EOF > ~/.ssh/config
            Host deploy-host
              Hostname $DEPLOY_HOST
              User $DEPLOY_USER
              IdentityFile ~/.ssh/deploy
              StrictHostKeyChecking no
            EOF
          '';
          effectScript = ''
            ${hci-effects.ssh { destination = "deploy-host"; } ''
              docker load < ${dockerImage}
              docker compose -p ${name} -f ${composeProjectFile} up -d
            ''}
          '';
        }
      )
    );
  };
}
