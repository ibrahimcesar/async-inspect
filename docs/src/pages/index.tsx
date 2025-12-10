import type {ReactNode} from 'react';
import clsx from 'clsx';
import Link from '@docusaurus/Link';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import Layout from '@theme/Layout';
import HomepageFeatures from '@site/src/components/HomepageFeatures';
import Heading from '@theme/Heading';
import CodeBlock from '@theme/CodeBlock';

import styles from './index.module.css';

function HomepageHeader() {
  const {siteConfig} = useDocusaurusContext();
  return (
    <header className={clsx('hero hero--primary', styles.heroBanner)}>
      <div className="container">
        <Heading as="h1" className="hero__title">
          {siteConfig.title}
        </Heading>
        <p className="hero__subtitle">{siteConfig.tagline}</p>
        <div className={styles.buttons}>
          <Link
            className="button button--secondary button--lg"
            to="/intro">
            Get Started
          </Link>
          <Link
            className="button button--outline button--lg"
            to="https://github.com/ibrahimcesar/async-inspect">
            GitHub
          </Link>
          <Link
            className="button button--outline button--lg"
            to="https://crates.io/crates/async-inspect">
            crates.io
          </Link>
        </div>
        <div className={styles.badges}>
          <img
            src="https://img.shields.io/crates/v/async-inspect.svg"
            alt="Crates.io version"
            className={styles.badge}
          />
          <img
            src="https://img.shields.io/docsrs/async-inspect"
            alt="docs.rs"
            className={styles.badge}
          />
          <img
            src="https://img.shields.io/crates/l/async-inspect.svg"
            alt="License"
            className={styles.badge}
          />
        </div>
      </div>
    </header>
  );
}

function InstallSection() {
  return (
    <section className={styles.installSection}>
      <div className="container">
        <Heading as="h2">Quick Install</Heading>
        <div className={styles.installCommand}>
          cargo add async-inspect
        </div>
      </div>
    </section>
  );
}

function CodeExampleSection() {
  const basicExample = `use async_inspect::prelude::*;

#[async_inspect::trace]
async fn fetch_user(id: u64) -> User {
    let profile = fetch_profile(id).await;
    let posts = fetch_posts(id).await;
    User { profile, posts }
}`;

  const deadlockExample = `use async_inspect::deadlock::DeadlockDetector;

// Automatically detect potential deadlocks
let detector = DeadlockDetector::new();
detector.start_monitoring();

// Get detailed diagnostics
if let Some(cycle) = detector.detect_cycle() {
    println!("Deadlock detected: {:?}", cycle);
}`;

  return (
    <section className={styles.codeExample}>
      <div className="container">
        <Heading as="h2" className={styles.codeExampleTitle}>
          Simple, Powerful API
        </Heading>
        <div className={styles.codeExampleContainer}>
          <div>
            <div className={styles.codeBlockLabel}>Trace async functions</div>
            <CodeBlock language="rust">{basicExample}</CodeBlock>
          </div>
          <div>
            <div className={styles.codeBlockLabel}>Detect deadlocks</div>
            <CodeBlock language="rust">{deadlockExample}</CodeBlock>
          </div>
        </div>
      </div>
    </section>
  );
}

export default function Home(): ReactNode {
  const {siteConfig} = useDocusaurusContext();
  return (
    <Layout
      title="X-ray vision for async Rust"
      description="Debugging tool for async Rust - inspect state machines, detect deadlocks, and analyze performance">
      <HomepageHeader />
      <main>
        <InstallSection />
        <HomepageFeatures />
        <CodeExampleSection />
      </main>
    </Layout>
  );
}
