import Heading from '@jetbrains/ring-ui-built/components/heading/heading';
import Island from '@jetbrains/ring-ui-built/components/island/island';
import IslandHeader from '@jetbrains/ring-ui-built/components/island/header';
import IslandContent from '@jetbrains/ring-ui-built/components/island/content';
import Text from '@jetbrains/ring-ui-built/components/text/text';

export const QueuePage = () => {
    return (
        <Island>
            <IslandHeader border>
                <Heading level={1}>Task Queue</Heading>
            </IslandHeader>
            <IslandContent>
                <Text>No tasks in queue</Text>
            </IslandContent>
        </Island>
    );
};
