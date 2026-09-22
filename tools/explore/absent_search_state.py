"""Observe explicitly absent saved-search containers and all recycler capacity.

All five null roots are a state classification, not proof of successful path
completion. The paired call result provides that meaning. No headers or owned
nodes are invented, and occupied recycler targets are not silently followed.
"""
from defined_search_graph import GraphObserver
from search_graph import POOLS,words
from replay_capsule import require


class AbsentSearchObserver(GraphObserver):
    def collect(self):
        unit=words(self.record(1,0,self.unit+0x104,72))
        require(not any(unit[:5]),'absent-search state requires all five roots null')
        capacities=[];lengths=[]
        for owner,address in enumerate(POOLS):
            header=words(self.record(6,owner,address,16))
            require(header[2]<=header[1]<=4096,'invalid pool bounds')
            if header[1]:self.record(7,owner,header[0],header[1]*4,header[2]*4)
            capacities.append(header[1]);lengths.append(header[2])
        return dict(frame=self.frame,records=len(self.rows),
                    declared_bytes=sum(len(d) for _,_,_,d in self.rows),
                    saved_containers='absent',recycler_capacities=capacities,
                    recycler_lengths=lengths,recycler_targets_observed=False)

    def run(self):
        result=super().run()
        result['schema']='model-absent-search-state-v1'
        return result
