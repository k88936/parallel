import {useLocation, useNavigate} from 'react-router-dom';

import Header, {Logo, Tray, HeaderIcon, Profile} from '@jetbrains/ring-ui-built/components/header/header';
import Link from '@jetbrains/ring-ui-built/components/link/link';
import Links from '@jetbrains/ring-ui-built/components/header/links';

import folderIcon from '@jetbrains/icons/folder-20px';
import agentsIcon from '@jetbrains/icons/agents-20px';
import hourglassIcon from '@jetbrains/icons/hourglass-20px';
import settingsIcon from '@jetbrains/icons/settings-20px';
import bellIcon from '@jetbrains/icons/bell-20px';
import helpIcon from '@jetbrains/icons/help-20px';

const PARALLEL_LOGO = `<svg xmlns="http://www.w3.org/2000/svg" width="40" height="40" viewBox="0 0 40 40">
  <rect width="40" height="40" rx="8" fill="#6B57FF"/>
  <path d="M12 10h16v4H16v5h10v4H16v7h-4V10z" fill="#fff"/>
</svg>`;

export const AppHeader = () => {
    const location = useLocation();
    const navigate = useNavigate();

    const isActive = (path: string) => location.pathname.startsWith(path);

    return (
        <Header
            className="sticky top-0"
            spaced={true}
            vertical
        >
            <Link href="/" className="flex items-center no-underline">
                <Logo glyph={PARALLEL_LOGO} />
            </Link>
            <Links>
                <HeaderIcon
                    icon={folderIcon}
                    title="Projects"
                    active={isActive('/projects')}
                    onClick={() => navigate('/projects/root')}
                />
                <HeaderIcon
                    icon={agentsIcon}
                    title="Agents"
                    active={isActive('/agents')}
                    onClick={() => navigate('/agents')}
                />
                <HeaderIcon
                    icon={hourglassIcon}
                    title="Queue"
                    active={isActive('/queue')}
                    onClick={() => navigate('/queue')}
                />
                <HeaderIcon
                    icon={settingsIcon}
                    title="Settings"
                    active={isActive('/settings')}
                    onClick={() => navigate('/settings')}
                />
            </Links>
            <Tray>
                <HeaderIcon
                    icon={bellIcon}
                    title="Notifications"
                />
                <HeaderIcon
                    icon={helpIcon}
                    title="Help"
                />
                <Profile
                    round
                    user={{
                        id: '1',
                        login: 'user',
                        name: 'User',
                    }}
                />
            </Tray>
        </Header>
    );
};
