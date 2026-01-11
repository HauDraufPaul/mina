import clsx from 'clsx';
import Link from '@docusaurus/Link';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import Layout from '@theme/Layout';
import Heading from '@theme/Heading';

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
            Get Started - 5min ⏱️
          </Link>
        </div>
      </div>
    </header>
  );
}

export default function Home(): JSX.Element {
  const {siteConfig} = useDocusaurusContext();
  return (
    <Layout
      title={`${siteConfig.title}`}
      description="MINA Documentation - Monitoring, Intelligence, Networking, Automation">
      <HomepageHeader />
      <main>
        <div className="container margin-vert--lg">
          <div className="row">
            <div className="col col--4 margin-bottom--lg">
              <div className="card">
                <div className="card__header">
                  <Heading as="h3">🚀 Quick Start</Heading>
                </div>
                <div className="card__body">
                  <p>Get up and running with MINA in minutes. Learn the basics and start automating.</p>
                  <Link to="/getting-started/overview">Get Started →</Link>
                </div>
              </div>
            </div>
            <div className="col col--4 margin-bottom--lg">
              <div className="card">
                <div className="card__header">
                  <Heading as="h3">⚡ Automation</Heading>
                </div>
                <div className="card__body">
                  <p>Create scripts and workflows to automate tasks. Learn how to build powerful automations.</p>
                  <Link to="/modules/automation-circuit">Learn More →</Link>
                </div>
              </div>
            </div>
            <div className="col col--4 margin-bottom--lg">
              <div className="card">
                <div className="card__header">
                  <Heading as="h3">📊 Intelligence</Heading>
                </div>
                <div className="card__body">
                  <p>Monitor markets, track portfolios, and analyze data with MINA's intelligence features.</p>
                  <Link to="/modules/market-intelligence">Explore →</Link>
                </div>
              </div>
            </div>
          </div>
        </div>
      </main>
    </Layout>
  );
}
