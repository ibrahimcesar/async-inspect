import type {ReactNode} from 'react';
import clsx from 'clsx';
import Heading from '@theme/Heading';
import styles from './styles.module.css';

type FeatureItem = {
  title: string;
  Svg: React.ComponentType<React.ComponentProps<'svg'>>;
  description: ReactNode;
};

const FeatureList: FeatureItem[] = [
  {
    title: 'State Machine Inspection',
    Svg: require('@site/static/img/feature-inspect.svg').default,
    description: (
      <>
        See exactly what your futures are doing. Inspect current state,
        captured variables, and await points in real-time. No more guessing
        why your async code is stuck.
      </>
    ),
  },
  {
    title: 'Deadlock Detection',
    Svg: require('@site/static/img/feature-deadlock.svg').default,
    description: (
      <>
        Automatically detect circular dependencies and potential deadlocks
        before they crash production. Get actionable diagnostics with
        task dependency graphs.
      </>
    ),
  },
  {
    title: 'Performance Profiling',
    Svg: require('@site/static/img/feature-performance.svg').default,
    description: (
      <>
        Identify bottlenecks with detailed timing analysis. Compare
        performance across versions, detect regressions, and export
        to Chrome DevTools and flamegraphs.
      </>
    ),
  },
  {
    title: 'Multi-Runtime Support',
    Svg: require('@site/static/img/feature-runtime.svg').default,
    description: (
      <>
        Works with Tokio, async-std, smol, and more. One API for all
        async runtimes with runtime-agnostic instrumentation and
        zero-cost abstractions.
      </>
    ),
  },
  {
    title: 'Real-Time Dashboard',
    Svg: require('@site/static/img/feature-dashboard.svg').default,
    description: (
      <>
        Monitor your async application with a live web dashboard or
        terminal UI. Track task states, resource usage, and events
        as they happen.
      </>
    ),
  },
  {
    title: 'Execution Timeline',
    Svg: require('@site/static/img/feature-timeline.svg').default,
    description: (
      <>
        Visualize task execution with interactive timelines. See when
        tasks start, await, resume, and complete. Perfect for debugging
        complex concurrent workflows.
      </>
    ),
  },
];

function Feature({title, Svg, description}: FeatureItem) {
  return (
    <div className={clsx('col col--4')}>
      <div className="text--center">
        <Svg className={styles.featureSvg} role="img" />
      </div>
      <div className="text--center padding-horiz--md">
        <Heading as="h3">{title}</Heading>
        <p>{description}</p>
      </div>
    </div>
  );
}

export default function HomepageFeatures(): ReactNode {
  return (
    <section className={styles.features}>
      <div className="container">
        <div className="row">
          {FeatureList.map((props, idx) => (
            <Feature key={idx} {...props} />
          ))}
        </div>
      </div>
    </section>
  );
}
