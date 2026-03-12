import type {ReactNode} from 'react';

import Island from '@jetbrains/ring-ui-built/components/island/island';
import IslandContent from "@jetbrains/ring-ui-built/components/island/content";
import IslandHeader from "@jetbrains/ring-ui-built/components/island/header";

interface SidebarProps {
    title: string;
    children: ReactNode;
    actions?: ReactNode;
    className?: string;
}

export const Sidebar = ({
                            title,
                            children,
                        }: SidebarProps) => {
    return (
        <Island className={"w-90 min-w-70 flex m-2 flex-col border rounded overflow-hidden"}>
                <IslandHeader>
                    {title}
                </IslandHeader>
            <IslandContent>{children}</IslandContent>
        </Island>
    );
};

export default Sidebar;
