import * as React from 'react';
import { Connection } from '../Connection';
import { Icon } from './Icon';
import { Types, Maybe } from '@dotstats/common';
import stable from 'stable';

import githubIcon from '../icons/mark-github.svg';
import './Chains.css';

interface ChainData {
  label: Types.ChainLabel;
  nodeCount: Types.NodeCount;
}

export namespace Chains {
  export interface Props {
    chains: Map<Types.ChainLabel, Types.NodeCount>,
    subscribed: Maybe<Types.ChainLabel>,
    connection: Promise<Connection>
  }
}

export class Chains extends React.Component<Chains.Props, {}> {
  public render() {
    return (
      <div className="Chains">
        {
          this.chains.map((chain) => this.renderChain(chain))
        }
        <a className="Chains-fork-me" href="https://github.com/paritytech/substrate-telemetry" target="_blank">
          <Icon src={githubIcon} alt="Fork Me!" />
        </a>
      </div>
    );
  }

  private renderChain(chain: ChainData): React.ReactNode {
    const { label, nodeCount } = chain;

    const className = label === this.props.subscribed
      ? 'Chains-chain Chains-chain-selected'
      : 'Chains-chain';

    return (
      <a key={label} className={className} onClick={this.subscribe.bind(this, label)}>
        {label} <span className="Chains-node-count" title="Node Count">{nodeCount}</span>
      </a>
    )
  }

  private get chains(): ChainData[] {
    return stable
      .inplace(
        Array.from(this.props.chains.entries()),
        (a, b) => {
          if (a[0] === 'Alexander') {
            return -1;
          }

          if (b[0] === 'Alexander') {
            return 1;
          }

          return b[1] - a[1];
        }
      )
      .map(([label, nodeCount]) => ({ label, nodeCount }));
  }

  private async subscribe(chain: Types.ChainLabel) {
    const connection = await this.props.connection;

    connection.subscribe(chain);
    connection.resetConsensus();
  }
}
