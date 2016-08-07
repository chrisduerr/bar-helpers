# Path to your oh-my-zsh installation.
ZSH=/usr/share/oh-my-zsh/


#####################
##### POWERLINE #####
#####################

ZSH_THEME="powerlevel9k/powerlevel9k"
POWERLEVEL9K_MODE='awesome-fontconfig'
POWERLEVEL9K_SHORTEN_DIR_LENGTH=2
POWERLEVEL9K_PROMPT_ON_NEWLINE=true
POWERLEVEL9K_LEFT_PROMPT_ELEMENTS=(os_icon background_jobs virtualenv dir vcs)
POWERLEVEL9K_RIGHT_PROMPT_ELEMENTS=()

POWERLEVEL9K_LEFT_SEGMENT_SEPARATOR=""
POWERLEVEL9K_LEFT_SUBSEGMENT_SEPARATOR="⸾"
POWERLEVEL9K_LEFT_SEGMENT_END_SEPARATOR=""
POWERLEVEL9K_MULTILINE_FIRST_PROMPT_PREFIX="┏"
POWERLEVEL9K_MULTILINE_SECOND_PROMPT_PREFIX="┗ "

POWERLEVEL9K_OS_ICON_BACKGROUND="008"
POWERLEVEL9K_OS_ICON_FOREGROUND="009"

POWERLEVEL9K_DIR_HOME_BACKGROUND="008"
POWERLEVEL9K_DIR_HOME_FOREGROUND="007"

POWERLEVEL9K_VCS_CLEAN_BACKGROUND="008"
POWERLEVEL9K_VCS_CLEAN_FOREGROUND="002"
POWERLEVEL9K_VCS_MODIFIED_BACKGROUND="008"
POWERLEVEL9K_VCS_MODIFIED_FOREGROUND="009"
POWERLEVEL9K_VCS_UNTRACKED_BACKGROUND="008"
POWERLEVEL9K_VCS_UNTRACKED_FOREGROUND="004"

POWERLEVEL9K_VIRTUALENV_BACKGROUND="008"
POWERLEVEL9K_VIRTUALENV_FOREGROUND="007"

POWERLEVEL9K_DIR_DEFAULT_BACKGROUND="008"
POWERLEVEL9K_DIR_DEFAULT_FOREGROUND="007"
POWERLEVEL9K_DIR_HOME_SUBFOLDER_BACKGROUND="008"
POWERLEVEL9K_DIR_HOME_SUBFOLDER_FOREGROUND="007"

POWERLEVEL9K_BACKGROUND_JOBS_BACKGROUND="008"
POWERLEVEL9K_BACKGROUND_JOBS_FOREGROUND="007"


# Uncomment the following line to disable bi-weekly auto-update checks.
DISABLE_AUTO_UPDATE="true"

# Plugins
plugins=(git)

source $ZSH/oh-my-zsh.sh

ZSH_CACHE_DIR=$HOME/.oh-my-zsh-cache
if [[ ! -d $ZSH_CACHE_DIR ]]; then
  mkdir $ZSH_CACHE_DIR
fi


####################
##### ENVSTUFF #####
####################

# Use vim for ssh instead of nvim
if [[ -n $SSH_CONNECTION ]]; then
  export EDITOR='vim'
else
  export EDITOR='nvim'
fi

export PATH=$HOME/.cargo/bin:$HOME/bin:/usr/local/bin:$PATH
export VISUAL="nvim"

# Set cd and ls folder colors
export LS_COLORS='ow=36:di=34:fi=32:ex=31:ln=35:'
zstyle ':completion:*:default' list-colors ${(s.:.)LS_COLORS}
zstyle ':completion:*:*:*:*:*' menu yes select


######################
##### ALIASSTUFF #####
######################

alias sudo="sudo "
alias sshul="ssh -p 6666 undeadleech@undeadleech.com"
alias leechnot="python2 ~/Scripts/leechnot.py"
alias sct='systemctl'
alias vim="nvim"
