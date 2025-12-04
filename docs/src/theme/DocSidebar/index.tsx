import React from 'react';
import DocSidebar from '@theme-original/DocSidebar';
import type DocSidebarType from '@theme/DocSidebar';
import type {WrapperProps} from '@docusaurus/types';

type Props = WrapperProps<typeof DocSidebarType>;

export default function DocSidebarWrapper(props: Props): JSX.Element {
  return (
    <>
      <DocSidebar {...props} />
      <div style={{
        padding: '16px',
        marginTop: '16px',
        borderTop: '1px solid var(--ifm-toc-border-color)',
      }}>
        <iframe
          src="https://github.com/sponsors/ibrahimcesar/card"
          title="Sponsor ibrahimcesar"
          height="225"
          width="100%"
          style={{border: 0, maxWidth: '600px'}}
        />
      </div>
    </>
  );
}
